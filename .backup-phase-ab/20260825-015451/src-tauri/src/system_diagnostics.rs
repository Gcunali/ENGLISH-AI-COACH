use crate::{
    local_ai_probe,
    models::{DEFAULT_OLLAMA_MODEL, DEFAULT_WHISPER_MODEL, DEFAULT_WHISPER_THREADS},
    paths::{LocalAiPaths, LocalPaths},
    pronunciation::{PRONUNCIATION_MODEL_ID, PRONUNCIATION_MODEL_REVISION},
    reliability::{
        self, DIAGNOSTIC_REPORT_VERSION, PLATFORM_RELIABILITY_SCHEMA_VERSION,
        STARTUP_RECOVERY_RULE_VERSION, SYSTEM_EVENT_SCHEMA_VERSION,
    },
};
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::{fs, path::Path};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticComponentDto {
    pub status: String,
    pub version: Option<String>,
    pub message: String,
    pub technical_code: Option<String>,
    pub advanced_details: Value,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemDiagnosticsDto {
    pub report_version: u32,
    pub generated_at: String,
    pub app_version: String,
    pub platform: String,
    pub database: DiagnosticComponentDto,
    pub ollama: DiagnosticComponentDto,
    pub whisper: DiagnosticComponentDto,
    pub piper: DiagnosticComponentDto,
    pub voice_bridge: DiagnosticComponentDto,
    pub voice_streaming: DiagnosticComponentDto,
    pub pronunciation: DiagnosticComponentDto,
    pub settings: DiagnosticComponentDto,
    pub overall_status: String,
    pub conversation_ready: bool,
    pub pronunciation_ready: bool,
    pub database_ready: bool,
}

pub async fn run(
    paths: &LocalPaths,
    local_ai: &LocalAiPaths,
    client: &Client,
) -> SystemDiagnosticsDto {
    let probe = local_ai_probe::run(local_ai, client).await;
    let database = match reliability::validate_database(&paths.db_file()) {
        Ok(value) => component(
            "healthy",
            Some(value.schema_version.to_string()),
            "Database integrity and foreign keys are OK.",
            None,
            serde_json::json!({"schemaVersion":value.schema_version,"integrity":value.integrity,"foreignKeyViolations":value.foreign_key_violations,"journalMode":journal_mode(&paths.db_file()),"platformReliabilitySchemaVersion":PLATFORM_RELIABILITY_SCHEMA_VERSION,"startupRecoveryRuleVersion":STARTUP_RECOVERY_RULE_VERSION,"systemEventSchemaVersion":SYSTEM_EVENT_SCHEMA_VERSION}),
        ),
        Err(error) => component(
            "unavailable",
            None,
            &error,
            Some("DB_INTEGRITY_FAILED"),
            serde_json::json!({"integrity":"failed"}),
        ),
    };
    let ollama = if !probe.ollama.reachable {
        component(
            "unavailable",
            None,
            "Ollama is not reachable on the local endpoint.",
            Some("OLLAMA_UNAVAILABLE"),
            serde_json::json!({"endpoint":"http://127.0.0.1:11434","generationInvoked":false}),
        )
    } else if !probe.ollama.model_found {
        component(
            "unavailable",
            None,
            "Ollama is running, but qwen3.5:4b is not installed.",
            Some("OLLAMA_MODEL_MISSING"),
            serde_json::json!({"model":DEFAULT_OLLAMA_MODEL,"generationInvoked":false}),
        )
    } else {
        component(
            "healthy",
            Some(DEFAULT_OLLAMA_MODEL.into()),
            "Local language model is available.",
            None,
            serde_json::json!({"endpoint":"http://127.0.0.1:11434","generationInvoked":false}),
        )
    };
    let whisper = if probe.whisper.cli_found && probe.whisper.model_found {
        component(
            "healthy",
            Some(DEFAULT_WHISPER_MODEL.into()),
            "Whisper executable and model are readable.",
            None,
            serde_json::json!({"threads":DEFAULT_WHISPER_THREADS,"cliPath":probe.whisper.cli_path,"modelPath":probe.whisper.model_path}),
        )
    } else {
        component(
            "unavailable",
            Some(DEFAULT_WHISPER_MODEL.into()),
            if !probe.whisper.cli_found {
                "Whisper executable is missing."
            } else {
                "Whisper model is missing."
            },
            Some(if !probe.whisper.cli_found {
                "WHISPER_EXECUTABLE_MISSING"
            } else {
                "WHISPER_MODEL_MISSING"
            }),
            serde_json::json!({"threads":DEFAULT_WHISPER_THREADS,"cliFound":probe.whisper.cli_found,"modelFound":probe.whisper.model_found}),
        )
    };
    let piper = if probe.piper.python_found
        && probe.piper.installed
        && probe.piper.voice_found
        && probe.piper.voice_config_found
    {
        component(
            "healthy",
            probe.piper.version.clone(),
            "Piper runtime, voice and configuration are available.",
            None,
            serde_json::json!({"voice":probe.piper.voice_name,"pythonPath":probe.piper.python_path,"voiceModelPath":probe.piper.voice_model_path}),
        )
    } else {
        component(
            "unavailable",
            probe.piper.version.clone(),
            "Piper runtime or en_US-lessac-medium voice is unavailable.",
            Some("PIPER_UNAVAILABLE"),
            serde_json::json!({"pythonFound":probe.piper.python_found,"packageFound":probe.piper.installed,"voiceFound":probe.piper.voice_found,"voiceConfigFound":probe.piper.voice_config_found}),
        )
    };
    let voice_bridge = file_component(
        &local_ai.voice_engine_bridge(),
        "Voice Bridge is available.",
        "Voice Bridge is missing.",
        "VOICE_BRIDGE_MISSING",
        None,
    );
    let voice_streaming = file_component(
        &local_ai.local_ai_root.join("voice_streaming_runtime.py"),
        "Voice Streaming Runtime v1 is available.",
        "Voice Streaming Runtime is missing.",
        "VOICE_STREAMING_MISSING",
        Some("1".into()),
    );
    let pronunciation = pronunciation_component(local_ai);
    let settings = settings_component(&paths.db_file());
    let database_ready = database.status == "healthy";
    let conversation_ready = database_ready
        && ollama.status == "healthy"
        && whisper.status == "healthy"
        && piper.status == "healthy"
        && voice_bridge.status == "healthy"
        && voice_streaming.status == "healthy";
    let pronunciation_ready =
        database_ready && whisper.status == "healthy" && pronunciation.status == "healthy";
    let overall_status = if conversation_ready && pronunciation_ready {
        "All systems ready"
    } else if !conversation_ready {
        "Core conversation unavailable"
    } else {
        "Some components need attention"
    }
    .into();
    SystemDiagnosticsDto {
        report_version: DIAGNOSTIC_REPORT_VERSION,
        generated_at: system_timestamp(),
        app_version: env!("CARGO_PKG_VERSION").into(),
        platform: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        database,
        ollama,
        whisper,
        piper,
        voice_bridge,
        voice_streaming,
        pronunciation,
        settings,
        overall_status,
        conversation_ready,
        pronunciation_ready,
        database_ready,
    }
}

pub fn sanitized_json(report: &SystemDiagnosticsDto) -> Result<String, String> {
    serde_json::to_string_pretty(report)
        .map_err(|e| format!("Could not export diagnostic report: {e}"))
}

fn component(
    status: &str,
    version: Option<String>,
    message: &str,
    code: Option<&str>,
    details: Value,
) -> DiagnosticComponentDto {
    DiagnosticComponentDto {
        status: status.into(),
        version,
        message: message.into(),
        technical_code: code.map(str::to_owned),
        advanced_details: details,
    }
}
fn file_component(
    path: &Path,
    ok: &str,
    bad: &str,
    code: &str,
    version: Option<String>,
) -> DiagnosticComponentDto {
    if path.is_file() {
        component(
            "healthy",
            version,
            ok,
            None,
            serde_json::json!({"path":path.display().to_string()}),
        )
    } else {
        component(
            "unavailable",
            version,
            bad,
            Some(code),
            serde_json::json!({"available":false}),
        )
    }
}
fn pronunciation_component(local_ai: &LocalAiPaths) -> DiagnosticComponentDto {
    let python = local_ai.pronunciation_python();
    let worker = local_ai.pronunciation_worker();
    let model = local_ai.pronunciation_model_dir();
    let manifest = local_ai
        .pronunciation_root()
        .join("pronunciation_model_manifest.json");
    let manifest_valid = fs::read(&manifest)
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .is_some_and(|v| {
            v.get("modelId").and_then(Value::as_str) == Some(PRONUNCIATION_MODEL_ID)
                && v.get("revision").and_then(Value::as_str) == Some(PRONUNCIATION_MODEL_REVISION)
        });
    let model_files = [
        "config.json",
        "preprocessor_config.json",
        "tokenizer_config.json",
        "vocab.json",
        "pytorch_model.bin",
    ]
    .iter()
    .all(|name| model.join(name).is_file());
    let ready = python.is_file() && worker.is_file() && manifest_valid && model_files;
    if ready {
        component(
            "healthy",
            Some("1".into()),
            "Pronunciation runtime and model manifest are ready.",
            None,
            serde_json::json!({"modelId":PRONUNCIATION_MODEL_ID,"revision":PRONUNCIATION_MODEL_REVISION,"manifestValidation":"metadata","fullHashValidation":"on demand"}),
        )
    } else {
        component(
            "unavailable",
            Some("1".into()),
            "Pronunciation runtime or model manifest is invalid.",
            Some("PRONUNCIATION_MODEL_INVALID"),
            serde_json::json!({"pythonFound":python.is_file(),"workerFound":worker.is_file(),"manifestValid":manifest_valid,"modelFilesPresent":model_files}),
        )
    }
}
fn settings_component(database: &Path) -> DiagnosticComponentDto {
    let result = (|| {
        let c = crate::database::open(database)?;
        let mut s = c
            .prepare("SELECT key,value_json FROM settings")
            .map_err(|e| e.to_string())?;
        let rows = s
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        let mut invalid = Vec::new();
        for row in rows {
            let (key, value) = row.map_err(|e| e.to_string())?;
            if matches!(
                key.as_str(),
                "use_learning_memory_in_lessons" | "use_streaming_voice_response"
            ) && !matches!(value.as_str(), "true" | "false")
            {
                invalid.push(key)
            }
        }
        Ok::<_, String>(invalid)
    })();
    match result {
        Ok(invalid) if invalid.is_empty() => component(
            "healthy",
            None,
            "Known settings are valid.",
            None,
            serde_json::json!({"invalidKeys":[]}),
        ),
        Ok(invalid) => component(
            "warning",
            None,
            "One or more known settings have invalid values.",
            Some("SETTINGS_INVALID"),
            serde_json::json!({"invalidKeys":invalid}),
        ),
        Err(error) => component(
            "warning",
            None,
            &error,
            Some("SETTINGS_CHECK_FAILED"),
            serde_json::json!({"available":false}),
        ),
    }
}
fn journal_mode(path: &Path) -> String {
    crate::database::open(path)
        .and_then(|c| {
            c.query_row("PRAGMA journal_mode", [], |r| r.get(0))
                .map_err(|e| e.to_string())
        })
        .unwrap_or_else(|_| "unknown".into())
}
fn system_timestamp() -> String {
    format!(
        "unix:{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn report_serialization_contains_no_personal_fixture() {
        let component = component(
            "healthy",
            None,
            "ready",
            None,
            serde_json::json!({"available":true}),
        );
        let report = SystemDiagnosticsDto {
            report_version: 1,
            generated_at: "now".into(),
            app_version: "test".into(),
            platform: "windows".into(),
            database: component.clone(),
            ollama: component.clone(),
            whisper: component.clone(),
            piper: component.clone(),
            voice_bridge: component.clone(),
            voice_streaming: component.clone(),
            pronunciation: component.clone(),
            settings: component,
            overall_status: "All systems ready".into(),
            conversation_ready: true,
            pronunciation_ready: true,
            database_ready: true,
        };
        let json = sanitized_json(&report).unwrap();
        for secret in [
            "known transcript fixture",
            "known profile text",
            "known vocabulary text",
            "known pronunciation target",
        ] {
            assert!(!json.contains(secret))
        }
    }
    #[test]
    #[ignore = "manual read-only diagnostics of the current development computer"]
    fn physical_system_diagnostics_report() {
        tauri::async_runtime::block_on(async {
            let root =
                std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA").expect("LOCALAPPDATA"))
                    .join("com.englishaicoach.desktop");
            let paths = LocalPaths::create(root).unwrap();
            let local_ai = LocalAiPaths::resolve();
            let client = crate::ollama::client().unwrap();
            let report = run(&paths, &local_ai, &client).await;
            println!("{}", sanitized_json(&report).unwrap());
            assert!(report.database_ready);
        })
    }
}
