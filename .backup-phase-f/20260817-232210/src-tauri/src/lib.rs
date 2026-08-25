mod database;
mod diagnostics;
mod lesson_analysis;
mod lesson_analysis_repository;
mod lesson_analyzer;
mod lesson_repository;
mod lesson_session;
mod local_ai_probe;
mod models;
mod ollama;
mod paths;
mod speech;
mod voice_engine;

use lesson_analysis::LessonAnalysis;
use lesson_analysis_repository::LessonAnalysisRepository;
use lesson_analyzer::LessonAnalyzer;
use lesson_repository::{
    CorrectionCandidate, Lesson, LessonRepository, LessonStatus, LessonSummary, NewLesson,
    TranscriptMessage,
};
use lesson_session::LessonSessionManager;
use local_ai_probe::LocalVoiceEngineProbe;
use models::{Diagnostics, OllamaModel, SpeechAudio, TimedText};
use paths::{LocalAiPaths, LocalPaths};
use reqwest::Client;
use serde::Serialize;
use tauri::{Manager, State};
use voice_engine::{VoiceEngineManager, VoiceEngineState, VoiceEngineStatus};

const VOICE_ENGINE_VERSION: &str = "voice_v2_bridge_v1";

struct AppState {
    paths: LocalPaths,
    local_ai: LocalAiPaths,
    client: Client,
    voice_engine: VoiceEngineManager,
    lessons: LessonSessionManager,
    analyzer: LessonAnalyzer,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StartLessonResult {
    lesson_id: String,
    lesson_status: LessonStatus,
    voice_engine_state: VoiceEngineState,
    lesson: Lesson,
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
async fn start_lesson(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<StartLessonResult, String> {
    let probe = local_ai_probe::run(&state.local_ai, &state.client).await;
    if !probe.offline_ready {
        return Err(format!(
            "Local voice engine is not ready: {}",
            probe.problems.join(" ")
        ));
    }
    state.voice_engine.ensure_available()?;
    let lesson = state.lessons.begin_lesson(&NewLesson {
        topic: None,
        mode: "free_conversation".to_owned(),
        whisper_model: probe.voice_defaults.whisper_model.to_owned(),
        whisper_threads: probe.voice_defaults.whisper_threads,
        ollama_model: probe.voice_defaults.ollama_model.to_owned(),
        piper_voice: probe.voice_defaults.piper_voice.to_owned(),
        voice_engine_version: VOICE_ENGINE_VERSION.to_owned(),
    })?;
    match state.voice_engine.start(
        app,
        &state.local_ai,
        lesson.id.clone(),
        state.lessons.clone(),
    ) {
        Ok(voice_engine) => Ok(StartLessonResult {
            lesson_id: lesson.id.clone(),
            lesson_status: lesson.status,
            voice_engine_state: voice_engine.state,
            lesson,
        }),
        Err(error) => {
            let _ = state.lessons.fail_lesson(&lesson.id, &error);
            Err(error)
        }
    }
}

#[tauri::command]
fn end_lesson(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<LessonSummary, String> {
    let lesson = state
        .lessons
        .get_active_lesson()?
        .ok_or_else(|| "There is no active lesson to end.".to_owned())?;
    if let Err(error) = state.voice_engine.stop(&app) {
        let _ = state.lessons.fail_lesson(&lesson.id, &error);
        return Err(error);
    }
    state.lessons.complete_lesson(&lesson.id)
}

#[tauri::command]
fn get_voice_engine_state(state: State<'_, AppState>) -> Result<VoiceEngineStatus, String> {
    state.voice_engine.get_state()
}

#[tauri::command]
fn get_active_lesson(state: State<'_, AppState>) -> Result<Option<Lesson>, String> {
    state.lessons.get_active_lesson()
}

#[tauri::command]
fn get_lesson(state: State<'_, AppState>, lesson_id: String) -> Result<Option<Lesson>, String> {
    state.lessons.get_lesson(&lesson_id)
}

#[tauri::command]
fn get_latest_completed_lesson(state: State<'_, AppState>) -> Result<Option<Lesson>, String> {
    state.lessons.get_latest_completed_lesson()
}

#[tauri::command]
fn get_lesson_transcript(
    state: State<'_, AppState>,
    lesson_id: String,
) -> Result<Vec<TranscriptMessage>, String> {
    state.lessons.get_messages(&lesson_id)
}

#[tauri::command]
fn get_lesson_corrections(
    state: State<'_, AppState>,
    lesson_id: String,
) -> Result<Vec<CorrectionCandidate>, String> {
    state.lessons.get_corrections(&lesson_id)
}

#[tauri::command]
fn get_lesson_analysis(
    state: State<'_, AppState>,
    lesson_id: String,
) -> Result<Option<LessonAnalysis>, String> {
    state.analyzer.get(&lesson_id)
}

#[tauri::command]
async fn analyze_lesson(
    state: State<'_, AppState>,
    lesson_id: String,
) -> Result<LessonAnalysis, String> {
    state.analyzer.analyze(&lesson_id).await
}

#[tauri::command]
async fn retry_lesson_analysis(
    state: State<'_, AppState>,
    lesson_id: String,
) -> Result<LessonAnalysis, String> {
    state.analyzer.retry(&lesson_id).await
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
            let lesson_repository = LessonRepository::new(paths.db_file());
            let recovered = lesson_repository
                .recover_stale_lessons()
                .map_err(std::io::Error::other)?;
            if recovered > 0 {
                log::info!("Recovered {recovered} unfinished lesson(s) as interrupted.");
            }
            let analysis_repository = LessonAnalysisRepository::new(paths.db_file());
            let recovered_analyses = analysis_repository
                .recover_interrupted()
                .map_err(std::io::Error::other)?;
            if recovered_analyses > 0 {
                log::info!(
                    "Recovered {recovered_analyses} interrupted lesson analysis request(s) as failed."
                );
            }
            let client = ollama::client().map_err(std::io::Error::other)?;
            let analyzer = LessonAnalyzer::new(
                lesson_repository.clone(),
                analysis_repository,
            )
            .map_err(std::io::Error::other)?;
            app.manage(AppState {
                paths,
                local_ai,
                client,
                voice_engine: VoiceEngineManager::default(),
                lessons: LessonSessionManager::new(lesson_repository),
                analyzer,
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                let state = window.state::<AppState>();
                state.voice_engine.shutdown();
                if let Err(error) = state.lessons.interrupt_active() {
                    log::error!("Could not preserve interrupted lesson on shutdown: {error}");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            diagnostics,
            probe_local_voice_engine,
            start_lesson,
            end_lesson,
            get_voice_engine_state,
            get_active_lesson,
            get_lesson,
            get_latest_completed_lesson,
            get_lesson_transcript,
            get_lesson_corrections,
            get_lesson_analysis,
            analyze_lesson,
            retry_lesson_analysis,
            list_ollama_models,
            transcribe_audio,
            chat_teacher,
            synthesize_speech
        ])
        .run(tauri::generate_context!())
        .expect("error while running English AI Coach");
}
