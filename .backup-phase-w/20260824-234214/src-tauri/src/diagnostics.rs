use crate::{
    local_ai_probe,
    models::{ComponentHealth, Diagnostics},
    paths::{LocalAiPaths, LocalPaths},
};
use reqwest::Client;

pub async fn run(paths: &LocalPaths, local_ai: &LocalAiPaths, client: &Client) -> Diagnostics {
    let probe = local_ai_probe::run(local_ai, client).await;
    let database_ready = crate::database::migrate(&paths.db_file()).is_ok();
    Diagnostics {
        components: vec![
            health(
                "ollama",
                "Ollama",
                probe.ollama.reachable,
                if probe.ollama.reachable {
                    "Local service ready".to_owned()
                } else {
                    "Not running on 127.0.0.1:11434".to_owned()
                },
            ),
            health(
                "llm",
                "Language model",
                probe.ollama.model_found,
                probe.ollama.model_name.to_owned(),
            ),
            health(
                "whisper",
                "Whisper",
                probe.whisper.cli_found && probe.whisper.model_found,
                format!(
                    "{} · {} threads",
                    probe.whisper.model_name, probe.whisper.threads
                ),
            ),
            health(
                "vad",
                "Voice activity detection",
                true,
                "RMS VAD configured; Silero is optional".to_owned(),
            ),
            health(
                "piper",
                "Piper voice",
                probe.piper.python_found
                    && probe.piper.installed
                    && probe.piper.voice_found
                    && probe.piper.voice_config_found,
                format!(
                    "{} · Piper {}",
                    probe.piper.voice_name,
                    probe.piper.version.as_deref().unwrap_or("not detected")
                ),
            ),
            health("microphone", "Microphone", false, "Not tested".to_owned()),
            health("speaker", "Speaker", false, "Not tested".to_owned()),
            health(
                "sqlite",
                "Local database",
                database_ready,
                if database_ready {
                    "Ready (WAL mode)"
                } else {
                    "Database unavailable"
                }
                .to_owned(),
            ),
        ],
        data_directory: probe.local_ai_root,
        offline_ready: probe.offline_ready,
        platform: std::env::consts::OS.to_owned(),
    }
}
fn health(name: &'static str, label: &'static str, ready: bool, detail: String) -> ComponentHealth {
    ComponentHealth {
        name,
        label,
        ready,
        detail,
        repair_hint: None,
    }
}
