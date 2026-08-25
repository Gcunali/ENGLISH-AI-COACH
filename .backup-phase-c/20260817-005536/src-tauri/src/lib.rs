mod database;
mod diagnostics;
mod local_ai_probe;
mod models;
mod ollama;
mod paths;
mod speech;

use local_ai_probe::LocalVoiceEngineProbe;
use models::{Diagnostics, OllamaModel, SpeechAudio, TimedText};
use paths::{LocalAiPaths, LocalPaths};
use reqwest::Client;
use tauri::{Manager, State};

struct AppState {
    paths: LocalPaths,
    local_ai: LocalAiPaths,
    client: Client,
}

#[tauri::command]
async fn diagnostics(state: State<'_, AppState>) -> Result<Diagnostics, String> {
    Ok(diagnostics::run(&state.paths, &state.local_ai, &state.client).await)
}

#[tauri::command]
async fn probe_local_voice_engine(
    state: State<'_, AppState>,
) -> Result<LocalVoiceEngineProbe, String> {
    Ok(local_ai_probe::run(&state.local_ai, &state.client).await)
}

#[tauri::command]
async fn list_ollama_models(state: State<'_, AppState>) -> Result<Vec<OllamaModel>, String> {
    ollama::list_models(&state.client).await
}

#[tauri::command]
async fn transcribe_audio(
    state: State<'_, AppState>,
    audio_base64: String,
) -> Result<TimedText, String> {
    speech::transcribe(state.paths.clone(), audio_base64).await
}

#[tauri::command]
async fn chat_teacher(
    state: State<'_, AppState>,
    text: String,
    model: String,
) -> Result<TimedText, String> {
    ollama::chat(&state.client, &state.paths.db_file(), &model, &text).await
}

#[tauri::command]
async fn synthesize_speech(
    state: State<'_, AppState>,
    text: String,
) -> Result<SpeechAudio, String> {
    speech::synthesize(state.paths.clone(), text).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            let root = app.path().app_local_data_dir().map_err(|error| {
                std::io::Error::other(format!("Local app data directory unavailable: {error}"))
            })?;
            let paths = LocalPaths::create(root).map_err(std::io::Error::other)?;
            let local_ai = LocalAiPaths::resolve();
            database::migrate(&paths.db_file()).map_err(std::io::Error::other)?;
            let client = ollama::client().map_err(std::io::Error::other)?;
            app.manage(AppState {
                paths,
                local_ai,
                client,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            diagnostics,
            probe_local_voice_engine,
            list_ollama_models,
            transcribe_audio,
            chat_teacher,
            synthesize_speech
        ])
        .run(tauri::generate_context!())
        .expect("error while running English AI Coach");
}
