use crate::{
    paths::LocalAiPaths,
    pronunciation::{
        validate_result, PronunciationEngineStatus, PronunciationResult,
        PRONUNCIATION_ENGINE_VERSION, PRONUNCIATION_MODEL_ID, PRONUNCIATION_MODEL_REVISION,
        PRONUNCIATION_RESULT_SCHEMA_VERSION, PRONUNCIATION_SCORE_VERSION,
    },
};
use serde::Deserialize;
use serde_json::json;
use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

#[derive(Clone)]
pub struct PronunciationEngineManager {
    inner: Arc<Mutex<Option<Worker>>>,
    cancelled: Arc<AtomicBool>,
    last_error: Arc<Mutex<Option<String>>>,
}
impl Default for PronunciationEngineManager {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
            cancelled: Arc::new(AtomicBool::new(false)),
            last_error: Arc::new(Mutex::new(None)),
        }
    }
}
struct Worker {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    ready: ReadyEvent,
    #[cfg(windows)]
    _job: ProcessJob,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadyEvent {
    event: String,
    engine_version: u32,
    score_version: u32,
    schema_version: u32,
    model_id: String,
    model_revision: String,
    model_manifest_hash: String,
    phonemizer_ready: bool,
    load_ms: u32,
}

impl PronunciationEngineManager {
    pub fn status(&self, paths: &LocalAiPaths, load: bool) -> PronunciationEngineStatus {
        let installed = paths.pronunciation_python().is_file()
            && paths.pronunciation_worker().is_file()
            && paths
                .pronunciation_model_dir()
                .join("pytorch_model.bin")
                .is_file();
        if load && installed {
            if let Err(e) = self.ensure_started(paths) {
                *self.last_error.lock().unwrap() = Some(e)
            }
        }
        let guard = self.inner.lock().unwrap();
        let ready = guard.as_ref().map(|w| w.ready.clone());
        PronunciationEngineStatus {
            installed,
            available: installed && self.last_error.lock().unwrap().is_none(),
            ready: ready.is_some(),
            engine_version: PRONUNCIATION_ENGINE_VERSION,
            score_version: PRONUNCIATION_SCORE_VERSION,
            result_schema_version: PRONUNCIATION_RESULT_SCHEMA_VERSION,
            model_id: PRONUNCIATION_MODEL_ID.into(),
            model_revision: PRONUNCIATION_MODEL_REVISION.into(),
            phonemizer_ready: ready.as_ref().is_some_and(|r| r.phonemizer_ready),
            load_ms: ready.map(|r| r.load_ms),
            last_error: self.last_error.lock().unwrap().clone(),
        }
    }
    fn ensure_started(&self, paths: &LocalAiPaths) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "Pronunciation worker lock failed.".to_owned())?;
        if guard
            .as_mut()
            .is_some_and(|w| w.child.try_wait().ok().flatten().is_none())
        {
            return Ok(());
        }
        *guard = None;
        let mut command = Command::new(paths.pronunciation_python());
        command
            .arg("-u")
            .arg(paths.pronunciation_worker())
            .env("PYTHONUTF8", "1")
            .env("TRANSFORMERS_OFFLINE", "1")
            .env("HF_HUB_OFFLINE", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .current_dir(paths.pronunciation_root());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command
            .spawn()
            .map_err(|e| format!("Could not start pronunciation worker: {e}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or("Pronunciation stdin unavailable.")?;
        let stdout = child
            .stdout
            .take()
            .ok_or("Pronunciation stdout unavailable.")?;
        #[cfg(windows)]
        let job = create_process_job(&child)?;
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("Could not read pronunciation readiness: {e}"))?;
        let ready: ReadyEvent = serde_json::from_str(&line)
            .map_err(|e| format!("Invalid pronunciation readiness: {e}"))?;
        if ready.event != "engine_ready"
            || ready.engine_version != 1
            || ready.score_version != 1
            || ready.schema_version != 1
            || ready.model_id != PRONUNCIATION_MODEL_ID
            || ready.model_revision != PRONUNCIATION_MODEL_REVISION
            || ready.model_manifest_hash.len() != 64
        {
            return Err("Pronunciation worker readiness metadata is invalid.".into());
        }
        *guard = Some(Worker {
            child,
            stdin,
            stdout: reader,
            ready,
            #[cfg(windows)]
            _job: job,
        });
        *self.last_error.lock().unwrap() = None;
        Ok(())
    }
    pub fn analyze(
        &self,
        paths: &LocalAiPaths,
        request_id: &str,
        target: &str,
        heard: &str,
        audio: &Path,
    ) -> Result<PronunciationResult, String> {
        self.cancelled.store(false, Ordering::SeqCst);
        self.ensure_started(paths)?;
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "Pronunciation worker lock failed.".to_owned())?;
        let worker = guard.as_mut().ok_or("Pronunciation worker unavailable.")?;
        let payload = json!({"command":"analyze","requestId":request_id,"targetText":target,"heardText":heard,"audioPath":audio});
        writeln!(worker.stdin, "{}", payload)
            .map_err(|e| format!("Could not send pronunciation request: {e}"))?;
        worker.stdin.flush().map_err(|e| e.to_string())?;
        loop {
            let mut line = String::new();
            if worker
                .stdout
                .read_line(&mut line)
                .map_err(|e| e.to_string())?
                == 0
            {
                *guard = None;
                return Err("Pronunciation worker stopped unexpectedly.".into());
            }
            let value: serde_json::Value = serde_json::from_str(&line)
                .map_err(|e| format!("Invalid pronunciation worker JSONL: {e}"))?;
            if value.get("requestId").and_then(|v| v.as_str()) != Some(request_id) {
                continue;
            }
            if value.get("event").and_then(|v| v.as_str()) == Some("request_error") {
                return Err(value
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Pronunciation analysis failed.")
                    .to_owned());
            }
            let mut result: PronunciationResult = serde_json::from_value(value)
                .map_err(|e| format!("Invalid pronunciation result: {e}"))?;
            if self.cancelled.load(Ordering::SeqCst) {
                result.status = "cancelled".into();
                result.overall_score = None;
                result.words.clear();
                result.issues.clear()
            }
            validate_result(&result, target)?;
            return Ok(result);
        }
    }
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst)
    }
    pub fn shutdown(&self) {
        self.cancel();
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(worker) = guard.as_mut() {
                let _ = writeln!(worker.stdin, "{}", json!({"command":"shutdown"}));
                let _ = worker.stdin.flush();
                for _ in 0..15 {
                    if worker.child.try_wait().ok().flatten().is_some() {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                let _ = worker.child.kill();
                let _ = worker.child.wait();
            }
            *guard = None;
        }
    }
}
impl Drop for PronunciationEngineManager {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) == 1 {
            self.shutdown()
        }
    }
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
#[cfg(windows)]
fn create_process_job(child: &Child) -> Result<ProcessJob, String> {
    use std::{mem::size_of, os::windows::io::AsRawHandle, ptr};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    unsafe {
        let handle = CreateJobObjectW(ptr::null(), ptr::null());
        if handle.is_null() {
            return Err("Could not create pronunciation Job Object.".into());
        }
        let job = ProcessJob(handle);
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if SetInformationJobObject(
            job.0,
            JobObjectExtendedLimitInformation,
            &info as *const _ as _,
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        ) == 0
        {
            return Err("Could not configure pronunciation Job Object.".into());
        }
        if AssignProcessToJobObject(job.0, child.as_raw_handle() as _) == 0 {
            return Err("Could not assign pronunciation worker to Job Object.".into());
        }
        Ok(job)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn manager_is_optional_and_not_voice_readiness() {
        let manager = PronunciationEngineManager::default();
        let paths =
            LocalAiPaths::from_project_root(std::env::temp_dir().join("missing-pronunciation"));
        let status = manager.status(&paths, false);
        assert!(!status.installed);
        assert!(!status.ready)
    }
}
