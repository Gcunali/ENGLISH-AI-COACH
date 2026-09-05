use crate::{
    guided_conversation::GuidedConversationRepository,
    lesson_repository::{CorrectionCandidate, TranscriptMessage},
    lesson_session::LessonSessionManager,
    paths::LocalAiPaths,
    voice_performance_repository::{VoicePerformanceRepository, VoiceTurnPerformanceDto},
};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter};

pub const VOICE_ENGINE_EVENT: &str = "voice-engine-event";
pub const LEARNING_CONTEXT_ENV: &str = "ENGLISH_AI_COACH_LEARNING_CONTEXT";
pub const LESSON_CONTEXT_ENV: &str = "ENGLISH_AI_COACH_LESSON_CONTEXT";
pub const STUDENT_PROFILE_CONTEXT_ENV: &str = "ENGLISH_AI_COACH_STUDENT_PROFILE_CONTEXT";
pub const STREAMING_ENABLED_ENV: &str = "ENGLISH_AI_COACH_STREAMING_ENABLED";
pub const TEMP_AUDIO_DIR_ENV: &str = "ENGLISH_AI_COACH_TEMP_AUDIO_DIR";
pub const GUIDED_CONFIG_ENV: &str = "ENGLISH_AI_COACH_GUIDED_CONFIG";

#[derive(Clone)]
pub struct GuidedVoiceSession {
    pub repository: GuidedConversationRepository,
    pub session_id: String,
    pub stage_id: String,
    pub config_json: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VoiceEngineEvent {
    EngineStarted,
    Calibrating,
    Calibrated {
        #[serde(rename = "voiceThreshold")]
        voice_threshold: f64,
    },
    Listening,
    StudentSpeaking,
    SpeechFinished,
    Transcribing,
    Transcript {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<TranscriptMessage>,
    },
    TeacherThinking,
    TeacherStreamStarted {
        #[serde(rename = "generationId")]
        generation_id: String,
    },
    TeacherResponseDelta {
        #[serde(rename = "generationId")]
        generation_id: String,
        delta: String,
        text: String,
    },
    TeacherChunkReady {
        #[serde(rename = "generationId")]
        generation_id: String,
        #[serde(rename = "chunkIndex")]
        chunk_index: u32,
    },
    TeacherPlaybackStarted {
        #[serde(rename = "generationId")]
        generation_id: String,
        #[serde(rename = "chunkIndex")]
        chunk_index: u32,
    },
    TeacherResponse {
        text: String,
        #[serde(
            default,
            rename = "generationId",
            skip_serializing_if = "Option::is_none"
        )]
        generation_id: Option<String>,
        #[serde(default)]
        partial: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<TranscriptMessage>,
        #[serde(
            default,
            rename = "correctionCandidate",
            skip_serializing_if = "Option::is_none"
        )]
        correction_candidate: Option<CorrectionCandidate>,
    },
    TeacherSpeaking {
        #[serde(
            default,
            rename = "generationId",
            skip_serializing_if = "Option::is_none"
        )]
        generation_id: Option<String>,
    },
    TeacherCancelRequested {
        requested: bool,
    },
    TeacherCancelled {
        #[serde(rename = "generationId")]
        generation_id: String,
        #[serde(rename = "deliveredText")]
        delivered_text: String,
    },
    StreamingFallback {
        #[serde(rename = "generationId")]
        generation_id: String,
        reason: String,
    },
    VoiceTurnMetrics {
        #[serde(flatten)]
        metrics: VoiceTurnPerformanceDto,
    },
    WhisperPerformance {
        #[serde(default, rename = "requestId")]
        request_id: Option<String>,
        #[serde(default)]
        generation: Option<u32>,
        #[serde(default, rename = "loadMs")]
        load_ms: Option<u32>,
        #[serde(default, rename = "inferenceMs")]
        inference_ms: Option<u32>,
        persistent: bool,
        #[serde(default)]
        fallback: bool,
    },
    TeacherFinished {
        #[serde(
            default,
            rename = "generationId",
            skip_serializing_if = "Option::is_none"
        )]
        generation_id: Option<String>,
    },
    Error {
        message: String,
        #[serde(default)]
        recoverable: bool,
    },
    EngineStopped,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceEngineState {
    #[default]
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceEngineStatus {
    pub state: VoiceEngineState,
    pub process_id: Option<u32>,
}

#[derive(Clone, Default)]
pub struct VoiceEngineManager {
    inner: Arc<Mutex<VoiceEngineRuntime>>,
}

#[derive(Default)]
struct VoiceEngineRuntime {
    state: VoiceEngineState,
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    process_job: Option<ProcessJob>,
    engine_stopped_emitted: bool,
    lesson_id: Option<String>,
    stop_requested: bool,
}

#[cfg(windows)]
struct ProcessJob(windows_sys::Win32::Foundation::HANDLE);

#[cfg(windows)]
unsafe impl Send for ProcessJob {}

#[cfg(windows)]
impl Drop for ProcessJob {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

#[cfg(not(windows))]
struct ProcessJob;

impl VoiceEngineManager {
    pub fn ensure_available(&self) -> Result<(), String> {
        let mut runtime = self.lock()?;
        ensure_start_available(&mut runtime)
    }

    pub fn start(
        &self,
        app: AppHandle,
        paths: &LocalAiPaths,
        lesson_id: String,
        lessons: LessonSessionManager,
        lesson_context: Option<&str>,
        student_profile_context: Option<&str>,
        learning_context: Option<&str>,
        streaming_enabled: bool,
        temp_audio_dir: &Path,
        performance: VoicePerformanceRepository,
        guided: Option<GuidedVoiceSession>,
    ) -> Result<VoiceEngineStatus, String> {
        let python = paths.piper_python();
        let bridge = paths.voice_engine_bridge();
        if !python.is_file() {
            return Err(format!(
                "Piper Python was not found at {}",
                python.display()
            ));
        }
        if !bridge.is_file() {
            return Err(format!(
                "Voice engine bridge was not found at {}",
                bridge.display()
            ));
        }

        let mut runtime = self.lock()?;
        ensure_start_available(&mut runtime)?;

        runtime.state = VoiceEngineState::Starting;
        runtime.engine_stopped_emitted = false;
        runtime.stop_requested = false;
        runtime.lesson_id = Some(lesson_id.clone());

        let mut command = Command::new(&python);
        command
            .arg("-u")
            .arg(&bridge)
            .current_dir(&paths.local_ai_root)
            .env("PYTHONUTF8", "1")
            .env("PYTHONUNBUFFERED", "1")
            .env_remove(LESSON_CONTEXT_ENV)
            .env_remove(STUDENT_PROFILE_CONTEXT_ENV)
            .env_remove(LEARNING_CONTEXT_ENV)
            .env_remove(GUIDED_CONFIG_ENV)
            .env(STREAMING_ENABLED_ENV, streaming_enabled.to_string())
            .env(TEMP_AUDIO_DIR_ENV, temp_audio_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(context) = lesson_context.filter(|value| !value.trim().is_empty()) {
            command.env(LESSON_CONTEXT_ENV, context);
        }
        if let Some(context) = student_profile_context.filter(|value| !value.trim().is_empty()) {
            command.env(STUDENT_PROFILE_CONTEXT_ENV, context);
        }
        if let Some(context) = learning_context.filter(|value| !value.trim().is_empty()) {
            command.env(LEARNING_CONTEXT_ENV, context);
        }
        if let Some(value) = guided.as_ref() {
            command.env(GUIDED_CONFIG_ENV, &value.config_json);
        }

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = command.spawn().map_err(|error| {
            runtime.state = VoiceEngineState::Error;
            runtime.lesson_id = None;
            format!("Could not start the local voice engine: {error}")
        })?;
        let process_id = child.id();
        let process_job = create_process_job(&child).map_err(|error| {
            let _ = child.kill();
            let _ = child.wait();
            runtime.state = VoiceEngineState::Error;
            runtime.lesson_id = None;
            error
        })?;
        let Some(stdin) = child.stdin.take() else {
            drop(process_job);
            let _ = child.kill();
            let _ = child.wait();
            runtime.state = VoiceEngineState::Error;
            runtime.lesson_id = None;
            return Err("Voice engine stdin was not captured.".to_owned());
        };
        let Some(stdout) = child.stdout.take() else {
            drop(process_job);
            let _ = child.kill();
            let _ = child.wait();
            runtime.state = VoiceEngineState::Error;
            runtime.lesson_id = None;
            return Err("Voice engine stdout was not captured.".to_owned());
        };
        let Some(stderr) = child.stderr.take() else {
            drop(process_job);
            let _ = child.kill();
            let _ = child.wait();
            runtime.state = VoiceEngineState::Error;
            runtime.lesson_id = None;
            return Err("Voice engine stderr was not captured.".to_owned());
        };
        runtime.child = Some(child);
        runtime.stdin = Some(stdin);
        runtime.process_job = Some(process_job);
        drop(runtime);

        self.read_stdout(
            app.clone(),
            process_id,
            lesson_id.clone(),
            lessons.clone(),
            performance,
            guided.clone(),
            stdout,
        );
        read_stderr(process_id, stderr);
        self.monitor(app, process_id, lesson_id, lessons, guided.is_some());

        Ok(VoiceEngineStatus {
            state: VoiceEngineState::Starting,
            process_id: Some(process_id),
        })
    }

    pub fn stop(&self, app: &AppHandle) -> Result<VoiceEngineStatus, String> {
        let mut runtime = self.lock()?;
        let Some(mut child) = runtime.child.take() else {
            runtime.stdin = None;
            runtime.state = VoiceEngineState::Stopped;
            return Ok(status(&runtime));
        };

        runtime.state = VoiceEngineState::Stopping;
        runtime.stop_requested = true;
        let process_id = child.id();
        if let Some(mut stdin) = runtime.stdin.take() {
            let _ = send_control(&mut stdin, "shutdown");
        }
        let mut process_job = runtime.process_job.take();
        if let Err(error) = stop_process_tree_gracefully(&mut child, &mut process_job) {
            runtime.child = Some(child);
            runtime.process_job = process_job;
            runtime.state = VoiceEngineState::Error;
            return Err(format!(
                "Could not stop voice engine process {process_id}: {error}"
            ));
        }

        let should_emit = !runtime.engine_stopped_emitted;
        runtime.engine_stopped_emitted = true;
        runtime.state = VoiceEngineState::Stopped;
        runtime.lesson_id = None;
        let current = status(&runtime);
        drop(runtime);

        if should_emit {
            emit(app, &VoiceEngineEvent::EngineStopped);
        }
        Ok(current)
    }

    pub fn cancel_current_response(&self) -> Result<bool, String> {
        let mut runtime = self.lock()?;
        let Some(stdin) = runtime.stdin.as_mut() else {
            return Ok(false);
        };
        send_control(stdin, "cancel_current_teacher_response")?;
        Ok(true)
    }

    pub fn get_state(&self) -> Result<VoiceEngineStatus, String> {
        let runtime = self.lock()?;
        Ok(status(&runtime))
    }

    pub fn shutdown(&self) {
        let Ok(mut runtime) = self.inner.lock() else {
            return;
        };
        if let Some(mut child) = runtime.child.take() {
            if let Some(mut stdin) = runtime.stdin.take() {
                let _ = send_control(&mut stdin, "shutdown");
            }
            let mut process_job = runtime.process_job.take();
            let _ = stop_process_tree_gracefully(&mut child, &mut process_job);
        }
        runtime.state = VoiceEngineState::Stopped;
        runtime.lesson_id = None;
        runtime.stop_requested = true;
    }

    fn read_stdout(
        &self,
        app: AppHandle,
        process_id: u32,
        lesson_id: String,
        lessons: LessonSessionManager,
        performance: VoicePerformanceRepository,
        guided: Option<GuidedVoiceSession>,
        stdout: std::process::ChildStdout,
    ) {
        let manager = self.clone();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                match line {
                    Ok(line) if line.trim().is_empty() => {}
                    Ok(line) => match parse_json_line(&line) {
                        Ok(mut event) => {
                            if guided.is_none() {
                                if let VoiceEngineEvent::VoiceTurnMetrics { metrics } = &event {
                                    if let Err(error) =
                                        performance.record(Some(&lesson_id), metrics)
                                    {
                                        log::error!(
                                            "Voice performance metric was not persisted: {error}"
                                        );
                                    }
                                }
                            }
                            let persisted = if let Some(sink) = &guided {
                                sink.repository.enrich_event(
                                    &sink.session_id,
                                    &sink.stage_id,
                                    &mut event,
                                )
                            } else {
                                lessons.enrich_event(&lesson_id, &mut event)
                            };
                            if let Err(error) = persisted {
                                let message = format!("Could not persist voice event: {error}");
                                log::error!("{message}");
                                if guided.is_none() {
                                    let _ = lessons.fail_lesson(&lesson_id, &message);
                                }
                                manager.observe_event(
                                    process_id,
                                    &VoiceEngineEvent::Error {
                                        message: message.clone(),
                                        recoverable: false,
                                    },
                                );
                                emit(
                                    &app,
                                    &VoiceEngineEvent::Error {
                                        message,
                                        recoverable: false,
                                    },
                                );
                                let _ = manager.stop(&app);
                                break;
                            }
                            manager.observe_event(process_id, &event);
                            emit(&app, &event);
                            if let VoiceEngineEvent::Error {
                                message,
                                recoverable: false,
                            } = &event
                            {
                                if guided.is_none() {
                                    let _ = lessons.fail_lesson(&lesson_id, message);
                                }
                                let _ = manager.stop(&app);
                                break;
                            }
                        }
                        Err(error) => {
                            log::warn!(
                                "Ignoring invalid JSONL from voice engine process {process_id}: {error}"
                            );
                        }
                    },
                    Err(error) => {
                        log::warn!(
                            "Could not read voice engine stdout for process {process_id}: {error}"
                        );
                        break;
                    }
                }
            }
        });
    }

    fn monitor(
        &self,
        app: AppHandle,
        process_id: u32,
        lesson_id: String,
        lessons: LessonSessionManager,
        guided: bool,
    ) {
        let manager = self.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(100));
            let outcome = {
                let Ok(mut runtime) = manager.inner.lock() else {
                    return;
                };
                let Some(child) = runtime.child.as_mut() else {
                    return;
                };
                if child.id() != process_id {
                    return;
                }
                match child.try_wait() {
                    Ok(Some(exit_status)) => {
                        let expected = runtime.stop_requested;
                        let already_reported_error = runtime.state == VoiceEngineState::Error;
                        let stopped_emitted = runtime.engine_stopped_emitted;
                        runtime.child = None;
                        runtime.stdin = None;
                        runtime.process_job = None;
                        runtime.lesson_id = None;
                        runtime.state = if expected || exit_status.success() {
                            VoiceEngineState::Stopped
                        } else {
                            VoiceEngineState::Error
                        };
                        Some((
                            exit_status,
                            expected,
                            already_reported_error,
                            stopped_emitted,
                        ))
                    }
                    Ok(None) => None,
                    Err(error) => {
                        log::warn!("Could not inspect voice engine process {process_id}: {error}");
                        None
                    }
                }
            };

            if let Some((exit_status, expected, already_reported_error, stopped_emitted)) = outcome
            {
                if !expected {
                    if !exit_status.success() || already_reported_error {
                        let message = format!(
                            "Voice engine process {process_id} exited unexpectedly ({exit_status})."
                        );
                        if !guided {
                            if let Err(error) = lessons.fail_lesson(&lesson_id, &message) {
                                log::error!("Could not mark lesson {lesson_id} as failed: {error}");
                            }
                        }
                    } else if !guided {
                        if let Err(error) = lessons.interrupt_lesson(&lesson_id) {
                            log::error!(
                                "Could not mark lesson {lesson_id} as interrupted: {error}"
                            );
                        }
                    }
                }
                if !expected && !exit_status.success() && !already_reported_error {
                    emit(
                        &app,
                        &VoiceEngineEvent::Error {
                            message: format!(
                                "Voice engine process {process_id} exited unexpectedly ({exit_status})."
                            ),
                            recoverable: false,
                        },
                    );
                }
                if !stopped_emitted {
                    emit(&app, &VoiceEngineEvent::EngineStopped);
                }
                return;
            }
        });
    }

    fn observe_event(&self, process_id: u32, event: &VoiceEngineEvent) {
        let Ok(mut runtime) = self.inner.lock() else {
            return;
        };
        if runtime.child.as_ref().map(Child::id) != Some(process_id) {
            return;
        }
        match event {
            VoiceEngineEvent::EngineStarted => runtime.state = VoiceEngineState::Running,
            VoiceEngineEvent::Error {
                recoverable: false, ..
            } => runtime.state = VoiceEngineState::Error,
            VoiceEngineEvent::EngineStopped => {
                runtime.state = VoiceEngineState::Stopping;
                runtime.engine_stopped_emitted = true;
            }
            _ => {}
        }
    }

    fn lock(&self) -> Result<MutexGuard<'_, VoiceEngineRuntime>, String> {
        self.inner
            .lock()
            .map_err(|_| "Voice engine state lock is unavailable.".to_owned())
    }
}

fn read_stderr(process_id: u32, stderr: std::process::ChildStderr) {
    thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            match line {
                Ok(line) if !line.trim().is_empty() => {
                    log::info!(target: "voice_engine", "[{process_id}] {line}");
                }
                Ok(_) => {}
                Err(error) => {
                    log::warn!(
                        "Could not read voice engine stderr for process {process_id}: {error}"
                    );
                    break;
                }
            }
        }
    });
}

fn parse_json_line(line: &str) -> Result<VoiceEngineEvent, String> {
    serde_json::from_str(line).map_err(|error| format!("{error}; line={line:?}"))
}

#[cfg(windows)]
fn create_process_job(child: &Child) -> Result<ProcessJob, String> {
    use std::{ffi::c_void, mem::size_of, os::windows::io::AsRawHandle, ptr};
    use windows_sys::Win32::{
        Foundation::GetLastError,
        System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
            SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        },
    };

    unsafe {
        let handle = CreateJobObjectW(ptr::null(), ptr::null());
        if handle.is_null() {
            return Err(format!(
                "Could not create the voice engine process job (Windows error {}).",
                GetLastError()
            ));
        }
        let job = ProcessJob(handle);
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if SetInformationJobObject(
            job.0,
            JobObjectExtendedLimitInformation,
            &limits as *const _ as *const c_void,
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        ) == 0
        {
            return Err(format!(
                "Could not configure the voice engine process job (Windows error {}).",
                GetLastError()
            ));
        }
        if AssignProcessToJobObject(job.0, child.as_raw_handle() as _) == 0 {
            return Err(format!(
                "Could not assign the voice engine to its process job (Windows error {}).",
                GetLastError()
            ));
        }
        Ok(job)
    }
}

#[cfg(not(windows))]
fn create_process_job(_child: &Child) -> Result<ProcessJob, String> {
    Ok(ProcessJob)
}

fn terminate_process_tree(
    child: &mut Child,
    process_job: &mut Option<ProcessJob>,
) -> Result<(), String> {
    // Closing the Windows job handle terminates only this app-created process tree.
    drop(process_job.take());
    if child
        .try_wait()
        .map_err(|error| format!("could not inspect it: {error}"))?
        .is_none()
    {
        child
            .kill()
            .map_err(|error| format!("could not terminate it: {error}"))?;
    }
    child
        .wait()
        .map_err(|error| format!("could not wait for it: {error}"))?;
    Ok(())
}

fn stop_process_tree_gracefully(
    child: &mut Child,
    process_job: &mut Option<ProcessJob>,
) -> Result<(), String> {
    for _ in 0..60 {
        if child
            .try_wait()
            .map_err(|error| format!("could not inspect it: {error}"))?
            .is_some()
        {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(25));
    }
    terminate_process_tree(child, process_job)
}

fn send_control(stdin: &mut ChildStdin, command_type: &str) -> Result<(), String> {
    let command = serde_json::json!({ "type": command_type });
    writeln!(stdin, "{command}")
        .and_then(|_| stdin.flush())
        .map_err(|error| format!("Could not send voice control command: {error}"))
}

fn ensure_start_available(runtime: &mut VoiceEngineRuntime) -> Result<(), String> {
    if let Some(child) = runtime.child.as_ref() {
        return Err(format!(
            "A voice engine session is already active (process {}).",
            child.id()
        ));
    }
    Ok(())
}

fn status(runtime: &VoiceEngineRuntime) -> VoiceEngineStatus {
    VoiceEngineStatus {
        state: runtime.state,
        process_id: runtime.child.as_ref().map(Child::id),
    }
}

fn emit(app: &AppHandle, event: &VoiceEngineEvent) {
    if let Err(error) = app.emit(VOICE_ENGINE_EVENT, event) {
        log::warn!("Could not emit {VOICE_ENGINE_EVENT}: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_start_available, parse_json_line, VoiceEngineEvent, VoiceEngineManager,
        VoiceEngineRuntime, VoiceEngineState,
    };
    use std::{
        process::{Child, Command, Stdio},
        thread,
        time::Duration,
    };

    #[test]
    fn parses_a_valid_state_event() {
        assert_eq!(
            parse_json_line(r#"{"type":"listening"}"#).unwrap(),
            VoiceEngineEvent::Listening
        );
    }

    #[test]
    fn parses_a_transcript_event() {
        assert_eq!(
            parse_json_line(r#"{"type":"transcript","text":"Hi teacher."}"#).unwrap(),
            VoiceEngineEvent::Transcript {
                text: "Hi teacher.".to_owned(),
                message: None,
            }
        );
    }

    #[test]
    fn parses_an_error_event() {
        assert_eq!(
            parse_json_line(r#"{"type":"error","message":"microphone unavailable"}"#).unwrap(),
            VoiceEngineEvent::Error {
                message: "microphone unavailable".to_owned(),
                recoverable: false,
            }
        );
    }

    #[test]
    fn parses_additive_streaming_and_metric_events() {
        assert_eq!(
            parse_json_line(r#"{"type":"teacher_response_delta","generationId":"turn-1","delta":"Hi","text":"Hi"}"#).unwrap(),
            VoiceEngineEvent::TeacherResponseDelta {
                generation_id: "turn-1".to_owned(),
                delta: "Hi".to_owned(),
                text: "Hi".to_owned(),
            }
        );
        let metric = parse_json_line(r#"{"type":"voice_turn_metrics","turnId":"turn-1","runtimeVersion":1,"streamingEnabled":true,"sttMs":100,"llmTtftMs":50,"llmFirstSentenceMs":200,"llmTotalMs":500,"firstTtsMs":120,"speechEndToFirstAudioMs":4000,"lastVoiceToFirstAudioMs":4000,"captureEndToFirstAudioMs":500,"ttsTotalMs":220,"teacherPlaybackMs":800,"teacherTurnTotalMs":1400,"ttsChunkCount":2,"cancelled":false,"fallbackUsed":false,"createdAt":"2026-08-22T00:00:00Z"}"#).unwrap();
        assert!(matches!(metric, VoiceEngineEvent::VoiceTurnMetrics { .. }));
    }

    #[test]
    fn recoverable_error_does_not_change_managed_process_to_error() {
        let child = long_running_child();
        let process_id = child.id();
        let manager = VoiceEngineManager::default();
        {
            let mut runtime = manager.inner.lock().unwrap();
            runtime.state = VoiceEngineState::Running;
            runtime.child = Some(child);
        }
        manager.observe_event(
            process_id,
            &VoiceEngineEvent::Error {
                message: "turn failed".to_owned(),
                recoverable: true,
            },
        );
        assert_eq!(
            manager.get_state().unwrap().state,
            VoiceEngineState::Running
        );
        manager.shutdown();
    }

    #[test]
    fn cancel_command_is_safe_when_idle_and_writes_to_an_active_process() {
        let manager = VoiceEngineManager::default();
        assert!(!manager.cancel_current_response().unwrap());
        let mut child = long_running_child_with_stdin();
        let stdin = child.stdin.take().unwrap();
        {
            let mut runtime = manager.inner.lock().unwrap();
            runtime.state = VoiceEngineState::Running;
            runtime.stdin = Some(stdin);
            runtime.child = Some(child);
        }
        assert!(manager.cancel_current_response().unwrap());
        manager.shutdown();
    }

    #[test]
    fn serializes_correction_candidate_with_the_frontend_field_name() {
        let event = VoiceEngineEvent::TeacherResponse {
            text: "You can say this.".to_owned(),
            generation_id: Some("generation-1".to_owned()),
            partial: false,
            message: None,
            correction_candidate: Some(crate::lesson_repository::CorrectionCandidate {
                id: "correction-1".to_owned(),
                lesson_id: "lesson-1".to_owned(),
                student_message_id: "student-1".to_owned(),
                teacher_message_id: "teacher-1".to_owned(),
                student_text: "student".to_owned(),
                teacher_response_text: "teacher".to_owned(),
                detection_method: "teacher_cue_v1".to_owned(),
                created_at: "now".to_owned(),
            }),
        };
        let serialized = serde_json::to_value(event).unwrap();
        assert!(serialized.get("correctionCandidate").is_some());
        assert!(serialized.get("correction_candidate").is_none());
    }

    #[test]
    fn rejects_an_invalid_line_without_panicking() {
        assert!(parse_json_line("Professor pensando...").is_err());
    }

    #[test]
    fn prevents_a_second_managed_process() {
        let mut child = long_running_child();
        let process_id = child.id();
        let mut runtime = VoiceEngineRuntime {
            state: VoiceEngineState::Running,
            child: Some(child),
            stdin: None,
            process_job: None,
            engine_stopped_emitted: false,
            lesson_id: Some("lesson".to_owned()),
            stop_requested: false,
        };

        let error = ensure_start_available(&mut runtime).unwrap_err();
        assert!(error.contains(&process_id.to_string()));

        child = runtime.child.take().unwrap();
        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    fn shutdown_kills_only_the_managed_process() {
        let managed = long_running_child();
        let mut unrelated = long_running_child();
        let manager = VoiceEngineManager::default();
        {
            let mut runtime = manager.inner.lock().unwrap();
            runtime.state = VoiceEngineState::Running;
            runtime.child = Some(managed);
            runtime.stdin = None;
            runtime.process_job = None;
        }

        manager.shutdown();
        let state = manager.get_state().unwrap();
        assert_eq!(state.state, VoiceEngineState::Stopped);
        assert_eq!(state.process_id, None);
        thread::sleep(Duration::from_millis(50));
        assert!(unrelated.try_wait().unwrap().is_none());

        let _ = unrelated.kill();
        let _ = unrelated.wait();
    }

    fn long_running_child() -> Child {
        #[cfg(windows)]
        let mut command = {
            let mut command = Command::new("ping.exe");
            command.args(["-n", "30", "127.0.0.1"]);
            command
        };
        #[cfg(not(windows))]
        let mut command = {
            let mut command = Command::new("sleep");
            command.arg("30");
            command
        };
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("long-running test process")
    }

    fn long_running_child_with_stdin() -> Child {
        #[cfg(windows)]
        let mut command = {
            let mut command = Command::new("ping.exe");
            command.args(["-n", "30", "127.0.0.1"]);
            command
        };
        #[cfg(not(windows))]
        let mut command = {
            let mut command = Command::new("sleep");
            command.arg("30");
            command
        };
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("long-running test process with stdin")
    }
}
