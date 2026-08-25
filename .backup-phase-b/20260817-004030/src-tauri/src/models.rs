use serde::Serialize;

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
