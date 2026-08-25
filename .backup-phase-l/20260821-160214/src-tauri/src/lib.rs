mod database;
mod diagnostics;
mod learning_memory_repository;
mod learning_repository;
mod lesson_analysis;
mod lesson_analysis_repository;
mod lesson_analyzer;
mod lesson_modes;
mod lesson_repository;
mod lesson_session;
mod local_ai_probe;
mod models;
mod ollama;
mod paths;
mod placement;
mod placement_bank;
mod placement_evaluator;
mod placement_repository;
mod placement_scoring;
mod placement_speech;
mod speech;
mod student_learning_summary_repository;
mod student_profile_repository;
mod voice_engine;

use learning_memory_repository::{
    LearningMemoryRepository, LearningMemorySummary, LearningMemorySyncResult,
    RecurringMistakeDetails, RecurringMistakeDto, VocabularyFilter, VocabularyItemDetails,
    VocabularyItemDto, VocabularyPage, VocabularySort, VocabularyStatus, VocabularySummary,
};
use learning_repository::{
    DashboardSummary, LearningRepository, LessonDetails, LessonHistoryFilter, LessonHistoryPage,
    ProgressOverview,
};
use lesson_analysis::LessonAnalysis;
use lesson_analysis_repository::LessonAnalysisRepository;
use lesson_analyzer::LessonAnalyzer;
use lesson_modes::{
    build_lesson_mode_context, lesson_modes, validate_start_request, LessonConfigurationDto,
    LessonConfigurationRepository, LessonModeDefinitionDto, LessonStartRequest,
};
use lesson_repository::{
    CorrectionCandidate, Lesson, LessonRepository, LessonStatus, LessonSummary, NewLesson,
    TranscriptMessage,
};
use lesson_session::LessonSessionManager;
use local_ai_probe::LocalVoiceEngineProbe;
use models::{Diagnostics, OllamaModel, SpeechAudio, TimedText};
use paths::{LocalAiPaths, LocalPaths};
use placement::{
    ConfirmSpeakingResponseRequest, PlacementAttemptDto, PlacementOverviewDto, PlacementResultDto,
    PlacementSessionDto, SubmitPlacementAnswerRequest,
};
use placement_evaluator::PlacementSpeakingEvaluator;
use placement_repository::{normalize_transcript, PlacementRepository};
use reqwest::Client;
use serde::Serialize;
use student_learning_summary_repository::{
    build_teacher_memory_context, StudentLearningSummary, StudentLearningSummaryRepository,
    TeacherMemorySnapshot, STUDENT_LEARNING_SUMMARY_SCHEMA_VERSION, TEACHER_MEMORY_CONTEXT_VERSION,
};
use student_profile_repository::{
    LessonStudentProfileSnapshotDto, StudentLearningProfileDto, StudentProfileContextStatusDto,
    StudentProfileRepository, UpdateStudentProfileRequest, STUDENT_LEARNING_PROFILE_SCHEMA_VERSION,
    STUDENT_PROFILE_CONTEXT_VERSION,
};
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
    learning: LearningRepository,
    memory: LearningMemoryRepository,
    summaries: StudentLearningSummaryRepository,
    configurations: LessonConfigurationRepository,
    placement: PlacementRepository,
    profiles: StudentProfileRepository,
    placement_evaluator: PlacementSpeakingEvaluator,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StartLessonResult {
    lesson_id: String,
    lesson_status: LessonStatus,
    voice_engine_state: VoiceEngineState,
    lesson: Lesson,
    memory_snapshot: TeacherMemorySnapshot,
    profile_snapshot: LessonStudentProfileSnapshotDto,
    configuration: LessonConfigurationDto,
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
    request: LessonStartRequest,
) -> Result<StartLessonResult, String> {
    let configuration = validate_start_request(request)?;
    let lesson_context = build_lesson_mode_context(&configuration)?;
    let probe = local_ai_probe::run(&state.local_ai, &state.client).await;
    if !probe.offline_ready {
        return Err(format!(
            "Local voice engine is not ready: {}",
            probe.problems.join(" ")
        ));
    }
    state.voice_engine.ensure_available()?;
    let lesson = state.lessons.begin_configured_lesson(
        &NewLesson {
            topic: configuration.topic.clone(),
            mode: configuration.mode_id.clone(),
            whisper_model: probe.voice_defaults.whisper_model.to_owned(),
            whisper_threads: probe.voice_defaults.whisper_threads,
            ollama_model: probe.voice_defaults.ollama_model.to_owned(),
            piper_voice: probe.voice_defaults.piper_voice.to_owned(),
            voice_engine_version: VOICE_ENGINE_VERSION.to_owned(),
        },
        &configuration,
    )?;
    let persisted_configuration = match state.configurations.get(&lesson.id) {
        Ok(Some(configuration)) => configuration,
        Ok(None) => {
            let error = "Lesson configuration snapshot could not be read back.".to_owned();
            let _ = state.lessons.fail_lesson(&lesson.id, &error);
            return Err(error);
        }
        Err(error) => {
            let _ = state.lessons.fail_lesson(&lesson.id, &error);
            return Err(error);
        }
    };
    let memory_enabled = match state.summaries.get_memory_enabled() {
        Ok(value) => value,
        Err(error) => {
            log::error!("Could not read learning memory setting; starting without memory: {error}");
            false
        }
    };
    let summary = match state.summaries.refresh_summary() {
        Ok(value) => Some(value),
        Err(error) => {
            log::error!("Could not refresh learning memory; starting without context: {error}");
            None
        }
    };
    let learning_context = if memory_enabled {
        summary.as_ref().and_then(build_teacher_memory_context)
    } else {
        None
    };
    let memory_snapshot = TeacherMemorySnapshot {
        enabled: memory_enabled,
        context_loaded: learning_context.is_some(),
        context_version: learning_context
            .as_ref()
            .map(|_| TEACHER_MEMORY_CONTEXT_VERSION),
        summary_schema_version: STUDENT_LEARNING_SUMMARY_SCHEMA_VERSION,
        analyzed_lesson_count_used: summary
            .as_ref()
            .map_or(0, |value| value.analyzed_lesson_count),
    };
    if let Err(error) = state
        .summaries
        .record_lesson_snapshot(&lesson.id, &memory_snapshot)
    {
        log::error!("Could not persist lesson memory metadata: {error}");
    }
    let prepared_profile = match state
        .profiles
        .prepare_for_lesson(&lesson.id, configuration.difficulty)
    {
        Ok(value) => value,
        Err(error) => {
            log::error!(
                "Could not build student profile; starting without profile context: {error}"
            );
            student_profile_repository::PreparedStudentProfile {
                context: None,
                snapshot: LessonStudentProfileSnapshotDto {
                    lesson_id: lesson.id.clone(),
                    profile_schema_version: STUDENT_LEARNING_PROFILE_SCHEMA_VERSION,
                    profile_context_version: STUDENT_PROFILE_CONTEXT_VERSION,
                    context_enabled: false,
                    placement_attempt_id: None,
                    estimated_cefr_level: None,
                    placement_confidence: None,
                    target_cefr_level: None,
                    learning_goals: Vec::new(),
                    lesson_difficulty: configuration.difficulty,
                    created_at: String::new(),
                },
            }
        }
    };
    if let Err(error) = state.profiles.record_snapshot(&prepared_profile.snapshot) {
        log::error!("Could not persist lesson student profile snapshot: {error}");
    }
    match state.voice_engine.start(
        app,
        &state.local_ai,
        lesson.id.clone(),
        state.lessons.clone(),
        Some(&lesson_context),
        prepared_profile.context.as_deref(),
        learning_context.as_deref(),
    ) {
        Ok(voice_engine) => Ok(StartLessonResult {
            lesson_id: lesson.id.clone(),
            lesson_status: lesson.status,
            voice_engine_state: voice_engine.state,
            lesson,
            memory_snapshot,
            profile_snapshot: prepared_profile.snapshot,
            configuration: persisted_configuration,
        }),
        Err(error) => {
            let _ = state.lessons.fail_lesson(&lesson.id, &error);
            Err(error)
        }
    }
}

#[tauri::command]
fn get_lesson_modes() -> Vec<LessonModeDefinitionDto> {
    lesson_modes()
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
fn get_dashboard_summary(state: State<'_, AppState>) -> Result<DashboardSummary, String> {
    state.learning.dashboard_summary()
}

#[tauri::command]
fn list_lessons(
    state: State<'_, AppState>,
    filter: LessonHistoryFilter,
    limit: u32,
    offset: u32,
) -> Result<LessonHistoryPage, String> {
    state.learning.list_lessons(filter, limit, offset)
}

#[tauri::command]
fn get_lesson_details(
    state: State<'_, AppState>,
    lesson_id: String,
) -> Result<Option<LessonDetails>, String> {
    state.learning.lesson_details(&lesson_id)
}

#[tauri::command]
fn get_progress_overview(state: State<'_, AppState>) -> Result<ProgressOverview, String> {
    state.learning.progress_overview()
}

#[tauri::command]
fn get_vocabulary_summary(state: State<'_, AppState>) -> Result<VocabularySummary, String> {
    state.memory.vocabulary_summary()
}

#[tauri::command]
fn get_learning_memory_summary(
    state: State<'_, AppState>,
) -> Result<LearningMemorySummary, String> {
    state.memory.summary()
}

#[tauri::command]
fn list_vocabulary_items(
    state: State<'_, AppState>,
    filter: VocabularyFilter,
    search: String,
    sort: VocabularySort,
    limit: u32,
    offset: u32,
) -> Result<VocabularyPage, String> {
    state
        .memory
        .list_vocabulary(filter, &search, sort, limit, offset)
}

#[tauri::command]
fn get_vocabulary_item(
    state: State<'_, AppState>,
    vocabulary_id: String,
) -> Result<Option<VocabularyItemDetails>, String> {
    state.memory.get_vocabulary_item(&vocabulary_id)
}

#[tauri::command]
fn update_vocabulary_status(
    state: State<'_, AppState>,
    vocabulary_id: String,
    status: VocabularyStatus,
) -> Result<VocabularyItemDto, String> {
    let item = state
        .memory
        .update_vocabulary_status(&vocabulary_id, status)?;
    if let Err(error) = state.summaries.refresh_summary() {
        log::error!("Vocabulary status changed, but learning summary refresh failed: {error}");
    }
    Ok(item)
}

#[tauri::command]
fn list_recurring_mistakes(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<RecurringMistakeDto>, String> {
    state.memory.list_recurring_mistakes(limit)
}

#[tauri::command]
fn get_recurring_mistake(
    state: State<'_, AppState>,
    mistake_id: String,
) -> Result<Option<RecurringMistakeDetails>, String> {
    state.memory.get_recurring_mistake(&mistake_id)
}

#[tauri::command]
fn sync_learning_memory(state: State<'_, AppState>) -> Result<LearningMemorySyncResult, String> {
    let result = state.memory.sync_all_completed_analyses()?;
    if let Err(error) = state.summaries.refresh_summary() {
        log::error!("Learning memory synchronized, but summary refresh failed: {error}");
    }
    Ok(result)
}

#[tauri::command]
fn get_student_learning_summary(
    state: State<'_, AppState>,
) -> Result<StudentLearningSummary, String> {
    state.summaries.refresh_summary()
}

#[tauri::command]
fn get_learning_memory_enabled(state: State<'_, AppState>) -> Result<bool, String> {
    state.summaries.get_memory_enabled()
}

#[tauri::command]
fn set_learning_memory_enabled(state: State<'_, AppState>, enabled: bool) -> Result<bool, String> {
    state.summaries.set_memory_enabled(enabled)
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
fn get_placement_overview(state: State<'_, AppState>) -> Result<PlacementOverviewDto, String> {
    state.placement.overview()
}

#[tauri::command]
fn start_placement_test(
    state: State<'_, AppState>,
    start_over: bool,
) -> Result<PlacementSessionDto, String> {
    state.placement.start(start_over)
}

#[tauri::command]
fn resume_placement_test(
    state: State<'_, AppState>,
    attempt_id: String,
) -> Result<PlacementSessionDto, String> {
    state.placement.session(&attempt_id)
}

#[tauri::command]
fn abandon_placement_test(
    state: State<'_, AppState>,
    attempt_id: String,
) -> Result<PlacementAttemptDto, String> {
    state.placement.abandon(&attempt_id)
}

#[tauri::command]
fn submit_placement_answer(
    state: State<'_, AppState>,
    request: SubmitPlacementAnswerRequest,
) -> Result<PlacementSessionDto, String> {
    state.placement.submit_answer(request)
}

#[tauri::command]
async fn capture_placement_speaking_response(
    state: State<'_, AppState>,
    audio_base64: String,
) -> Result<TimedText, String> {
    let transcription =
        placement_speech::transcribe(state.paths.clone(), state.local_ai.clone(), audio_base64)
            .await?;
    let text = normalize_transcript(&transcription.text)?;
    Ok(TimedText {
        text,
        elapsed_ms: transcription.elapsed_ms,
    })
}

#[tauri::command]
fn confirm_placement_speaking_response(
    state: State<'_, AppState>,
    request: ConfirmSpeakingResponseRequest,
) -> Result<PlacementSessionDto, String> {
    state.placement.confirm_speaking(request)
}

#[tauri::command]
fn skip_placement_speaking(
    state: State<'_, AppState>,
    attempt_id: String,
) -> Result<PlacementSessionDto, String> {
    state.placement.skip_speaking(&attempt_id)
}

#[tauri::command]
async fn finalize_placement_test(
    state: State<'_, AppState>,
    attempt_id: String,
) -> Result<PlacementResultDto, String> {
    let evaluation = if state.placement.has_minimum_speaking_data(&attempt_id)? {
        let samples = state.placement.speaking_samples(&attempt_id)?;
        match state.placement_evaluator.evaluate(&samples).await {
            Ok(value) => Some(value),
            Err(error) => {
                log::error!("Placement speaking evaluation unavailable: {error}");
                None
            }
        }
    } else {
        None
    };
    state.placement.finalize(&attempt_id, evaluation)
}

#[tauri::command]
fn get_placement_result(
    state: State<'_, AppState>,
    attempt_id: String,
) -> Result<Option<PlacementResultDto>, String> {
    state.placement.result(&attempt_id)
}

#[tauri::command]
fn get_current_placement_result(
    state: State<'_, AppState>,
) -> Result<Option<PlacementResultDto>, String> {
    state.placement.current_result()
}

#[tauri::command]
fn list_placement_attempts(state: State<'_, AppState>) -> Result<Vec<PlacementAttemptDto>, String> {
    state.placement.list_attempts()
}

#[tauri::command]
fn get_student_learning_profile(
    state: State<'_, AppState>,
) -> Result<StudentLearningProfileDto, String> {
    state.profiles.get()
}

#[tauri::command]
fn update_student_learning_profile(
    state: State<'_, AppState>,
    request: UpdateStudentProfileRequest,
) -> Result<StudentLearningProfileDto, String> {
    state.profiles.update(request)
}

#[tauri::command]
fn get_student_profile_context_status(
    state: State<'_, AppState>,
) -> Result<StudentProfileContextStatusDto, String> {
    state.profiles.context_status()
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
            let memory = LearningMemoryRepository::new(paths.db_file());
            match memory.sync_all_completed_analyses() {
                Ok(result) if result.failed > 0 => log::error!(
                    "Learning memory startup sync completed with {} failure(s): {}",
                    result.failed,
                    result.errors.join(" | ")
                ),
                Ok(result) if result.synchronized > 0 => log::info!(
                    "Synchronized learning memory from {} completed analysis record(s).",
                    result.synchronized
                ),
                Ok(_) => {}
                Err(error) => log::error!("Learning memory startup sync failed: {error}"),
            }
            let summaries = StudentLearningSummaryRepository::new(paths.db_file());
            if let Err(error) = summaries.refresh_summary() {
                log::error!("Student learning summary startup refresh failed: {error}");
            }
            let analyzer = LessonAnalyzer::new(
                lesson_repository.clone(),
                analysis_repository,
                memory.clone(),
                summaries.clone(),
            )
            .map_err(std::io::Error::other)?;
            let learning = LearningRepository::new(paths.db_file());
            let configurations = LessonConfigurationRepository::new(paths.db_file());
            let placement =
                PlacementRepository::new(paths.db_file()).map_err(std::io::Error::other)?;
            let profiles = StudentProfileRepository::new(paths.db_file(), placement.clone());
            let placement_evaluator =
                PlacementSpeakingEvaluator::new().map_err(std::io::Error::other)?;
            app.manage(AppState {
                paths,
                local_ai,
                client,
                voice_engine: VoiceEngineManager::default(),
                lessons: LessonSessionManager::new(lesson_repository),
                analyzer,
                learning,
                memory,
                summaries,
                configurations,
                placement,
                profiles,
                placement_evaluator,
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
            get_lesson_modes,
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
            get_dashboard_summary,
            list_lessons,
            get_lesson_details,
            get_progress_overview,
            get_vocabulary_summary,
            get_learning_memory_summary,
            list_vocabulary_items,
            get_vocabulary_item,
            update_vocabulary_status,
            list_recurring_mistakes,
            get_recurring_mistake,
            sync_learning_memory,
            get_student_learning_summary,
            get_learning_memory_enabled,
            set_learning_memory_enabled,
            get_placement_overview,
            start_placement_test,
            resume_placement_test,
            abandon_placement_test,
            submit_placement_answer,
            capture_placement_speaking_response,
            confirm_placement_speaking_response,
            skip_placement_speaking,
            finalize_placement_test,
            get_placement_result,
            get_current_placement_result,
            list_placement_attempts,
            get_student_learning_profile,
            update_student_learning_profile,
            get_student_profile_context_status,
            list_ollama_models,
            transcribe_audio,
            chat_teacher,
            synthesize_speech
        ])
        .run(tauri::generate_context!())
        .expect("error while running English AI Coach");
}
