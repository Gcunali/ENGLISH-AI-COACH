use crate::paths::LocalAiPaths;
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter};

pub const VOICE_ENGINE_EVENT: &str = "voice-engine-event";

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
    },
    TeacherThinking,
    TeacherResponse {
        text: String,
    },
    TeacherSpeaking,
    TeacherFinished,
    Error {
        message: String,
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
    process_job: Option<ProcessJob>,
    engine_stopped_emitted: bool,
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
    pub fn start(&self, app: AppHandle, paths: &LocalAiPaths) -> Result<VoiceEngineStatus, String> {
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

        let mut command = Command::new(&python);
        command
            .arg("-u")
            .arg(&bridge)
            .current_dir(&paths.local_ai_root)
            .env("PYTHONUTF8", "1")
            .env("PYTHONUNBUFFERED", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = command.spawn().map_err(|error| {
            runtime.state = VoiceEngineState::Error;
            format!("Could not start the local voice engine: {error}")
        })?;
        let process_id = child.id();
        let process_job = create_process_job(&child).map_err(|error| {
            let _ = child.kill();
            let _ = child.wait();
            runtime.state = VoiceEngineState::Error;
            error
        })?;
        let Some(stdout) = child.stdout.take() else {
            drop(process_job);
            let _ = child.kill();
            let _ = child.wait();
            runtime.state = VoiceEngineState::Error;
            return Err("Voice engine stdout was not captured.".to_owned());
        };
        let Some(stderr) = child.stderr.take() else {
            drop(process_job);
            let _ = child.kill();
            let _ = child.wait();
            runtime.state = VoiceEngineState::Error;
            return Err("Voice engine stderr was not captured.".to_owned());
        };
        runtime.child = Some(child);
        runtime.process_job = Some(process_job);
        drop(runtime);

        self.read_stdout(app.clone(), process_id, stdout);
        read_stderr(process_id, stderr);
        self.monitor(app, process_id);

        Ok(VoiceEngineStatus {
            state: VoiceEngineState::Starting,
            process_id: Some(process_id),
        })
    }

    pub fn stop(&self, app: &AppHandle) -> Result<VoiceEngineStatus, String> {
        let mut runtime = self.lock()?;
        let Some(mut child) = runtime.child.take() else {
            runtime.state = VoiceEngineState::Stopped;
            return Ok(status(&runtime));
        };

        runtime.state = VoiceEngineState::Stopping;
        let process_id = child.id();
        let mut process_job = runtime.process_job.take();
        if let Err(error) = terminate_process_tree(&mut child, &mut process_job) {
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
        let current = status(&runtime);
        drop(runtime);

        if should_emit {
            emit(app, &VoiceEngineEvent::EngineStopped);
        }
        Ok(current)
    }

    pub fn get_state(&self) -> Result<VoiceEngineStatus, String> {
        let mut runtime = self.lock()?;
        clear_finished_child(&mut runtime)?;
        Ok(status(&runtime))
    }

    pub fn shutdown(&self) {
        let Ok(mut runtime) = self.inner.lock() else {
            return;
        };
        if let Some(mut child) = runtime.child.take() {
            let mut process_job = runtime.process_job.take();
            let _ = terminate_process_tree(&mut child, &mut process_job);
        }
        runtime.state = VoiceEngineState::Stopped;
    }

    fn read_stdout(&self, app: AppHandle, process_id: u32, stdout: std::process::ChildStdout) {
        let manager = self.clone();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                match line {
                    Ok(line) if line.trim().is_empty() => {}
                    Ok(line) => match parse_json_line(&line) {
                        Ok(event) => {
                            manager.observe_event(process_id, &event);
                            emit(&app, &event);
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

    fn monitor(&self, app: AppHandle, process_id: u32) {
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
                        let expected = runtime.state == VoiceEngineState::Stopping;
                        let already_reported_error = runtime.state == VoiceEngineState::Error;
                        let stopped_emitted = runtime.engine_stopped_emitted;
                        runtime.child = None;
                        runtime.process_job = None;
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
                if !expected && !exit_status.success() && !already_reported_error {
                    emit(
                        &app,
                        &VoiceEngineEvent::Error {
                            message: format!(
                                "Voice engine process {process_id} exited unexpectedly ({exit_status})."
                            ),
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
            VoiceEngineEvent::Error { .. } => runtime.state = VoiceEngineState::Error,
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

fn clear_finished_child(runtime: &mut VoiceEngineRuntime) -> Result<(), String> {
    let Some(child) = runtime.child.as_mut() else {
        return Ok(());
    };
    if child
        .try_wait()
        .map_err(|error| format!("Could not inspect voice engine process: {error}"))?
        .is_some()
    {
        runtime.child = None;
        runtime.process_job = None;
        if runtime.state != VoiceEngineState::Error {
            runtime.state = VoiceEngineState::Stopped;
        }
    }
    Ok(())
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

fn ensure_start_available(runtime: &mut VoiceEngineRuntime) -> Result<(), String> {
    clear_finished_child(runtime)?;
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
                text: "Hi teacher.".to_owned()
            }
        );
    }

    #[test]
    fn parses_an_error_event() {
        assert_eq!(
            parse_json_line(r#"{"type":"error","message":"microphone unavailable"}"#).unwrap(),
            VoiceEngineEvent::Error {
                message: "microphone unavailable".to_owned()
            }
        );
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
            process_job: None,
            engine_stopped_emitted: false,
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
}
