use serde::Serialize;

pub const DEFAULT_WHISPER_MODEL: &str = "ggml-small.en-q5_1.bin";
pub const DEFAULT_WHISPER_THREADS: u16 = 12;
pub const DEFAULT_OLLAMA_MODEL: &str = "qwen3.5:4b";
pub const DEFAULT_PIPER_VOICE: &str = "en_US-lessac-medium";
pub const OPTIONAL_WHISPER_MODELS: [&str; 2] = ["ggml-base.en.bin", "ggml-small.en.bin"];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentHealth {
    pub name: &'static str,
    pub label: &'static str,
    pub ready: bool,
    pub detail: String,
    pub repair_hint: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub components: Vec<ComponentHealth>,
    pub data_directory: String,
    pub offline_ready: bool,
    pub platform: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimedText {
    pub text: String,
    pub elapsed_ms: u128,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechAudio {
    pub audio_base64: String,
    pub mime_type: &'static str,
    pub elapsed_ms: u128,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
}
