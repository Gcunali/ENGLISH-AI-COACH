use crate::{
    models::{ComponentHealth, Diagnostics},
    ollama,
    paths::{display_name, LocalPaths},
};
use reqwest::Client;

pub async fn run(paths: &LocalPaths, client: &Client) -> Diagnostics {
    let ollama_ready = ollama::health(client).await;
    let models = if ollama_ready {
        ollama::list_models(client).await.unwrap_or_default()
    } else {
        Vec::new()
    };
    let llm = models
        .iter()
        .find(|model| model.name.starts_with("qwen3.5:4b"))
        .or_else(|| models.first());
    let whisper_exe = paths.whisper_executable();
    let vad = paths.vad_model().is_file();
    let piper_exe = paths.piper_executable();
    let voice = paths.piper_voice();
    let whisper_ready = whisper_exe.is_some() && paths.whisper_model().is_file();
    let piper_ready =
        piper_exe.is_some() && voice.is_file() && voice.with_extension("onnx.json").is_file();
    let database_ready = crate::database::migrate(&paths.db_file()).is_ok();
    let offline_ready =
        ollama_ready && llm.is_some() && whisper_ready && vad && piper_ready && database_ready;
    Diagnostics {
        components: vec![
            health(
                "ollama",
                "Ollama",
                ollama_ready,
                if ollama_ready {
                    "Local service ready".to_owned()
                } else {
                    "Not running on 127.0.0.1:11434".to_owned()
                },
                "Install or start Ollama.",
            ),
            health(
                "llm",
                "Language model",
                llm.is_some(),
                llm.map(|model| model.name.clone())
                    .unwrap_or_else(|| "qwen3.5:4b not installed".to_owned()),
                "Download a model explicitly in Local Models.",
            ),
            health(
                "whisper",
                "Whisper",
                whisper_ready,
                whisper_exe
                    .as_ref()
                    .map(|path| display_name(path))
                    .unwrap_or_else(|| "whisper-cli.exe not found".to_owned()),
                "Run scripts/setup-windows.ps1.",
            ),
            health(
                "vad",
                "Silero VAD",
                vad,
                if vad {
                    "ggml-silero-v6.2.0.bin"
                } else {
                    "VAD model missing"
                }
                .to_owned(),
                "Download the small Silero VAD model.",
            ),
            health(
                "piper",
                "Piper voice",
                piper_ready,
                piper_exe
                    .as_ref()
                    .map(|path| display_name(path))
                    .unwrap_or_else(|| "piper.exe not found".to_owned()),
                "Install Piper and an English voice.",
            ),
            health(
                "microphone",
                "Microphone",
                true,
                "Permission checked when lesson starts".to_owned(),
                "Allow microphone access in Windows Privacy settings.",
            ),
            health(
                "speaker",
                "Speaker",
                true,
                "WebView audio output".to_owned(),
                "Select a Windows output device.",
            ),
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
                "Check folder permissions and free disk space.",
            ),
        ],
        data_directory: paths.root.display().to_string(),
        offline_ready,
        platform: std::env::consts::OS.to_owned(),
    }
}
fn health(
    name: &'static str,
    label: &'static str,
    ready: bool,
    detail: String,
    hint: &str,
) -> ComponentHealth {
    ComponentHealth {
        name,
        label,
        ready,
        detail,
        repair_hint: (!ready).then(|| hint.to_owned()),
    }
}
