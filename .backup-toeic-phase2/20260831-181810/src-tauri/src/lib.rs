mod curriculum;
mod database;
mod diagnostics;
mod gamification;
mod gamification_repository;
mod guided_conversation;
mod guided_learning_integration;
mod guided_lesson_audio;
mod interactive_exercise;
mod interactive_lesson;
mod interactive_lesson_analysis;
mod interactive_lesson_content;
mod interactive_lesson_engine;
mod interactive_lesson_repository;
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
mod practice_repository;
mod pronunciation;
mod pronunciation_engine;
mod pronunciation_repository;
mod pronunciation_speech;
mod reliability;
mod review;
mod review_repository;
mod sha256;
mod speech;
mod student_learning_summary_repository;
mod student_profile_repository;
mod system_diagnostics;
mod toeic;
mod toeic_item_bank;
mod ux_preferences;
mod voice_engine;
mod voice_performance_repository;

use base64::{engine::general_purpose::STANDARD, Engine};
use curriculum::{CurriculumCatalogDto, CurriculumRegistry, CurriculumService};
use gamification_repository::{
    AchievementDto, GamificationOverviewDto, GamificationProfileDto, GamificationRepository,
    GamificationSyncResult,
};
use guided_conversation::{
    GuidedConversationRepository, GUIDED_CONVERSATION_FINAL_GUARDRAIL, GUIDED_CONVERSATION_POLICY,
};
use guided_learning_integration::{GuidedLearningIntegrationRepository, GuidedPracticeTimeResult};
use guided_lesson_audio::{GuidedAudioDto, GuidedLessonAudioRuntime, StaticTtsCacheStatusDto};
use interactive_exercise::{SelectExerciseAttemptRequest, SubmitExerciseAttemptRequest};
use interactive_lesson::{
    GuidedLessonOverviewDto, GuidedPlaybackRequest, GuidedPlaybackSource,
    GuidedPronunciationRequest, InteractiveLessonDetailDto, InteractiveLessonSessionDto,
    InteractiveLessonSummaryDto, SelectGuidedAttemptRequest, StageActionRequest,
    StartInteractiveLessonRequest,
};
use interactive_lesson_analysis::{
    InteractiveAnalysisRequest, InteractiveLessonAnalysisDto, InteractiveLessonAnalysisService,
};
use interactive_lesson_content::InteractiveLessonContentRegistry;
use interactive_lesson_engine::InteractiveLessonEngine;
use interactive_lesson_repository::InteractiveLessonRepository;
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
use practice_repository::{
    CompletePracticeItemRequest, PracticeAvailabilityDto, PracticeItemResultDto,
    PracticeRepository, PracticeSessionDto, StartPracticeRequest,
};
use pronunciation::{
    validate_result, validate_target as validate_pronunciation_target, AnalyzePronunciationRequest,
    PronunciationAttemptDto, PronunciationEngineStatus,
};
use pronunciation_engine::PronunciationEngineManager;
use pronunciation_repository::PronunciationRepository;
use reliability::{
    BackupStatusDto, BackupSummaryDto, BackupValidationDto, ReliabilityManager,
    RestoreScheduledDto, SystemEventDto,
};
use reqwest::Client;
use review_repository::{
    ReviewOverviewDto, ReviewQueuePreviewDto, ReviewRepository, ReviewSessionDto,
    ReviewSessionSummaryDto, ReviewSubmitResult, StartReviewSessionRequest,
    SubmitReviewItemRequest,
};
use serde::Serialize;
use student_learning_summary_repository::{
    build_teacher_memory_context, StudentLearningSummary, StudentLearningSummaryRepository,
    TeacherMemorySnapshot, STUDENT_LEARNING_SUMMARY_SCHEMA_VERSION, TEACHER_MEMORY_CONTEXT_VERSION,
};
use student_profile_repository::{
    build_student_profile_context, LessonStudentProfileSnapshotDto, StudentLearningProfileDto,
    StudentProfileContextStatusDto, StudentProfileRepository, UpdateStudentProfileRequest,
    STUDENT_LEARNING_PROFILE_SCHEMA_VERSION, STUDENT_PROFILE_CONTEXT_VERSION,
};
use system_diagnostics::SystemDiagnosticsDto;
use tauri::{Emitter, Manager, State};
use toeic::{
    ToeicOverviewDto, ToeicRepository, ToeicResultDto, ToeicReviewItemDto, ToeicSessionDto,
    ToeicSubmitAnswerRequest,
};
use toeic_item_bank::ToeicItemBank;
use ux_preferences::WelcomeStateDto;
use voice_engine::{GuidedVoiceSession, VoiceEngineManager, VoiceEngineState, VoiceEngineStatus};
use voice_performance_repository::VoicePerformanceRepository;

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
    gamification: GamificationRepository,
    review: ReviewRepository,
    practice: PracticeRepository,
    voice_performance: VoicePerformanceRepository,
    pronunciation_engine: PronunciationEngineManager,
    pronunciation: PronunciationRepository,
    reliability: ReliabilityManager,
    guided_lessons: InteractiveLessonEngine,
    guided_audio: GuidedLessonAudioRuntime,
    guided_conversations: GuidedConversationRepository,
    guided_analysis: InteractiveLessonAnalysisService,
    guided_learning: GuidedLearningIntegrationRepository,
    curriculum: CurriculumService,
    toeic: ToeicRepository,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ToeicAudioDto {
    playback_id: String,
    audio_base64: String,
    mime_type: String,
    source: String,
    duration_ms: u64,
    runtime_version: u32,
    presentation_id: Option<String>,
    initial: bool,
    item_id: String,
    item_version: u32,
}

#[tauri::command]
fn get_toeic_overview(state: State<'_, AppState>) -> Result<ToeicOverviewDto, String> {
    state.toeic.overview()
}

#[tauri::command]
fn start_toeic_session(
    state: State<'_, AppState>,
    form_id: String,
    form_version: u32,
) -> Result<ToeicSessionDto, String> {
    state.toeic.start(&form_id, form_version)
}

#[tauri::command]
fn get_toeic_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<ToeicSessionDto, String> {
    state.toeic.session(&session_id)
}

#[tauri::command]
fn submit_toeic_answer(
    state: State<'_, AppState>,
    request: ToeicSubmitAnswerRequest,
) -> Result<ToeicSessionDto, String> {
    state.toeic.submit(request)
}

#[tauri::command]
fn advance_toeic_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<ToeicSessionDto, String> {
    state.toeic.advance(&session_id)
}

#[tauri::command]
fn abandon_toeic_session(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    state.guided_audio.cancel_active();
    state.toeic.abandon(&session_id)
}

#[tauri::command]
fn record_toeic_active_time(
    state: State<'_, AppState>,
    session_id: String,
    event_id: String,
    duration_seconds: u32,
) -> Result<u32, String> {
    state
        .toeic
        .record_active_time(&session_id, &event_id, duration_seconds)
}

#[tauri::command]
fn get_toeic_result(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<ToeicResultDto, String> {
    state.toeic.result(&session_id)
}

#[tauri::command]
fn get_toeic_review(
    state: State<'_, AppState>,
    session_id: String,
    mistakes_only: bool,
) -> Result<Vec<ToeicReviewItemDto>, String> {
    state.toeic.review(&session_id, mistakes_only)
}

#[tauri::command]
fn list_toeic_history(
    state: State<'_, AppState>,
) -> Result<Vec<toeic::ToeicHistoryEntryDto>, String> {
    state.toeic.history()
}

#[tauri::command]
async fn prepare_toeic_audio(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<ToeicAudioDto, String> {
    let context = state.toeic.begin_audio(&session_id)?;
    let request = GuidedPlaybackRequest {
        session_id,
        stage_id: "toeic-part1".into(),
        item_id: context.item_id.clone(),
    };
    let prepared = state
        .guided_audio
        .prepare(
            state.paths.clone(),
            state.local_ai.clone(),
            request,
            GuidedPlaybackSource {
                text: context.script,
                asset_id: None,
                package_hash: "toeic-bank-v1".into(),
            },
        )
        .await;
    let audio = match prepared {
        Ok(audio) => audio,
        Err(error) => {
            state
                .toeic
                .interrupt_audio(context.presentation_id.as_deref())?;
            return Err(error);
        }
    };
    Ok(ToeicAudioDto {
        playback_id: audio.playback_id,
        audio_base64: audio.audio_base64,
        mime_type: audio.mime_type,
        source: audio.source,
        duration_ms: audio.duration_ms,
        runtime_version: audio.runtime_version,
        presentation_id: context.presentation_id,
        initial: context.initial,
        item_id: context.item_id,
        item_version: context.item_version,
    })
}

#[tauri::command]
fn complete_toeic_audio(
    state: State<'_, AppState>,
    playback_id: String,
    session_id: String,
    item_id: String,
    item_version: u32,
    presentation_id: Option<String>,
) -> Result<ToeicSessionDto, String> {
    let request = GuidedPlaybackRequest {
        session_id: session_id.clone(),
        stage_id: "toeic-part1".into(),
        item_id: item_id.clone(),
    };
    state
        .guided_audio
        .confirm_completed(&playback_id, &request)?;
    state.toeic.complete_audio(
        &session_id,
        &item_id,
        item_version,
        presentation_id.as_deref(),
    )?;
    state.toeic.session(&session_id)
}

#[tauri::command]
fn cancel_toeic_audio(
    state: State<'_, AppState>,
    playback_id: String,
    presentation_id: Option<String>,
) -> Result<bool, String> {
    let cancelled = state.guided_audio.cancel(&playback_id);
    state.toeic.interrupt_audio(presentation_id.as_deref())?;
    Ok(cancelled)
}

fn integrate_completed_guided(
    app: &tauri::AppHandle,
    state: &AppState,
    session_id: &str,
) -> Result<(), String> {
    let result = state.guided_learning.sync_completed(session_id)?;
    if result.integrated_sessions > 0 {
        if let Err(error) = state.summaries.refresh_summary() {
            log::error!("Guided learning summary refresh failed: {error}");
        }
        let _ = app.emit("english-ai-coach:review-changed", ());
        match state.gamification.sync() {
            Ok(sync) => {
                let _ = app.emit("english-ai-coach:gamification-changed", &sync);
            }
            Err(error) => log::error!("Guided gamification sync failed: {error}"),
        }
    }
    Ok(())
}

#[tauri::command]
fn get_practice_availability(
    state: State<'_, AppState>,
) -> Result<PracticeAvailabilityDto, String> {
    state.practice.availability()
}

#[tauri::command]
fn start_practice_session(
    state: State<'_, AppState>,
    request: StartPracticeRequest,
) -> Result<PracticeSessionDto, String> {
    state.practice.start(request)
}

#[tauri::command]
fn get_practice_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<PracticeSessionDto, String> {
    state.practice.get(&session_id)
}

#[tauri::command]
fn record_practice_time(
    state: State<'_, AppState>,
    session_id: String,
    event_id: String,
    duration_seconds: u32,
) -> Result<u32, String> {
    state
        .practice
        .record_time(&session_id, &event_id, duration_seconds)
}

#[tauri::command]
fn complete_practice_item(
    state: State<'_, AppState>,
    request: CompletePracticeItemRequest,
) -> Result<PracticeItemResultDto, String> {
    state.practice.complete_item(request)
}

#[tauri::command]
fn complete_practice_session(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    session_id: String,
) -> Result<PracticeSessionDto, String> {
    let result = state.practice.complete(&session_id)?;
    match state.gamification.sync() {
        Ok(sync) => {
            let _ = app.emit("english-ai-coach:gamification-changed", &sync);
        }
        Err(error) => log::error!("Practice gamification sync failed: {error}"),
    }
    Ok(result)
}

#[tauri::command]
fn abandon_practice_session(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    state.practice.abandon(&session_id)
}

#[tauri::command]
fn get_course_catalog(state: State<'_, AppState>) -> Result<CurriculumCatalogDto, String> {
    state.curriculum.catalog()
}

#[tauri::command]
fn get_guided_lesson_overview(
    state: State<'_, AppState>,
) -> Result<GuidedLessonOverviewDto, String> {
    state.guided_lessons.overview()
}
#[tauri::command]
fn list_guided_lessons(state: State<'_, AppState>) -> Vec<InteractiveLessonSummaryDto> {
    state.guided_lessons.list()
}
#[tauri::command]
fn get_guided_lesson(
    state: State<'_, AppState>,
    lesson_id: String,
) -> Option<InteractiveLessonDetailDto> {
    state.guided_lessons.detail(&lesson_id)
}
#[tauri::command]
fn get_active_guided_lesson_session(
    state: State<'_, AppState>,
) -> Result<Option<InteractiveLessonSessionDto>, String> {
    state.guided_lessons.active()
}
#[tauri::command]
fn start_guided_lesson(
    state: State<'_, AppState>,
    request: StartInteractiveLessonRequest,
) -> Result<InteractiveLessonSessionDto, String> {
    state.guided_lessons.start(request)
}
#[tauri::command]
fn resume_guided_lesson(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<InteractiveLessonSessionDto, String> {
    state.guided_lessons.resume(&session_id)
}
#[tauri::command]
fn get_guided_lesson_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Option<InteractiveLessonSessionDto>, String> {
    state.guided_lessons.get_session(&session_id)
}

#[tauri::command]
async fn start_guided_conversation(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: StageActionRequest,
) -> Result<InteractiveLessonSessionDto, String> {
    let probe = local_ai_probe::run(&state.local_ai, &state.client).await;
    if !probe.offline_ready {
        return Err(format!(
            "Local voice engine is not ready: {}",
            probe.problems.join(" ")
        ));
    }
    state.voice_engine.ensure_available()?;
    let prepared = state
        .guided_conversations
        .prepare(&request.session_id, &request.stage_id)?;
    let profile = state.profiles.get()?;
    let profile_context = if profile.use_profile_in_lessons {
        Some(build_student_profile_context(
            &profile,
            profile.default_lesson_difficulty,
        )?)
    } else {
        None
    };
    let memory_context = if state.summaries.get_memory_enabled().unwrap_or(false) {
        state
            .summaries
            .refresh_summary()
            .ok()
            .as_ref()
            .and_then(build_teacher_memory_context)
    } else {
        None
    };
    let retry_last_student = prepared
        .history
        .last()
        .is_some_and(|item| item.role == "student");
    let config = serde_json::json!({
        "schemaVersion": 1,
        "policy": GUIDED_CONVERSATION_POLICY,
        "lessonContext": prepared.lesson_context,
        "finalGuardrail": GUIDED_CONVERSATION_FINAL_GUARDRAIL,
        "history": prepared.history,
        "alreadyStarted": prepared.already_started,
        "retryLastStudent": retry_last_student,
        "owner": prepared.owner,
    })
    .to_string();
    let streaming = state.voice_performance.streaming_enabled().unwrap_or(true);
    state.voice_engine.start(
        app,
        &state.local_ai,
        prepared.owner.clone(),
        state.lessons.clone(),
        None,
        profile_context.as_deref(),
        memory_context.as_deref(),
        streaming,
        &state.paths.temporary_audio,
        state.voice_performance.clone(),
        Some(GuidedVoiceSession {
            repository: state.guided_conversations.clone(),
            session_id: request.session_id.clone(),
            stage_id: request.stage_id.clone(),
            config_json: config,
        }),
    )?;
    state
        .guided_lessons
        .get_session(&request.session_id)?
        .ok_or("Guided Lesson session not found.".into())
}

#[tauri::command]
fn stop_guided_conversation(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<VoiceEngineStatus, String> {
    state.voice_engine.stop(&app)
}

#[tauri::command]
fn finish_guided_conversation(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: StageActionRequest,
) -> Result<InteractiveLessonSessionDto, String> {
    if state.voice_engine.get_state()?.state != VoiceEngineState::Stopped {
        return Err("Stop the active voice operation before finishing the conversation.".into());
    }
    state
        .guided_conversations
        .finish(&request.session_id, &request.stage_id)?;
    integrate_completed_guided(&app, &state, &request.session_id)?;
    state
        .guided_lessons
        .get_session(&request.session_id)?
        .ok_or("Guided Lesson session not found.".into())
}
#[tauri::command]
fn complete_guided_lesson_stage(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: StageActionRequest,
) -> Result<InteractiveLessonSessionDto, String> {
    let session_id = request.session_id.clone();
    let result = state.guided_lessons.complete(request)?;
    integrate_completed_guided(&app, &state, &session_id)?;
    state.guided_audio.cleanup_session(&session_id);
    Ok(result)
}
#[tauri::command]
fn skip_guided_lesson_stage(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: StageActionRequest,
) -> Result<InteractiveLessonSessionDto, String> {
    let session_id = request.session_id.clone();
    let result = state.guided_lessons.skip(request)?;
    integrate_completed_guided(&app, &state, &session_id)?;
    Ok(result)
}
#[tauri::command]
fn abandon_guided_lesson(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<InteractiveLessonSessionDto, String> {
    let result = state.guided_lessons.abandon(&session_id)?;
    state.guided_audio.cleanup_session(&session_id);
    Ok(result)
}
#[tauri::command]
fn list_recent_guided_lesson_sessions(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<InteractiveLessonSessionDto>, String> {
    state.guided_lessons.recent(limit)
}

#[tauri::command]
fn get_guided_lesson_analysis(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Option<InteractiveLessonAnalysisDto>, String> {
    state.guided_analysis.get(&session_id)
}

#[tauri::command]
async fn analyze_guided_lesson(
    state: State<'_, AppState>,
    request: InteractiveAnalysisRequest,
) -> Result<InteractiveLessonAnalysisDto, String> {
    state.guided_analysis.analyze(&request).await
}

#[tauri::command]
async fn retry_guided_lesson_conversation_analysis(
    state: State<'_, AppState>,
    request: InteractiveAnalysisRequest,
) -> Result<InteractiveLessonAnalysisDto, String> {
    state.guided_analysis.retry_conversation(&request).await
}

#[tauri::command]
fn finalize_guided_lesson_analysis(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: InteractiveAnalysisRequest,
) -> Result<InteractiveLessonAnalysisDto, String> {
    let result = state.guided_analysis.finalize(&request)?;
    integrate_completed_guided(&app, &state, &request.session_id)?;
    Ok(result)
}

#[tauri::command]
fn record_guided_practice_time(
    state: State<'_, AppState>,
    session_id: String,
    event_id: String,
    duration_seconds: u32,
) -> Result<GuidedPracticeTimeResult, String> {
    state
        .guided_learning
        .record_active_practice(&session_id, &event_id, duration_seconds)
}

#[tauri::command]
async fn prepare_guided_lesson_audio(
    state: State<'_, AppState>,
    request: GuidedPlaybackRequest,
) -> Result<GuidedAudioDto, String> {
    let source = state.guided_lessons.playback_source(&request)?;
    state
        .guided_audio
        .prepare(state.paths.clone(), state.local_ai.clone(), request, source)
        .await
}

#[tauri::command]
fn complete_guided_lesson_audio(
    state: State<'_, AppState>,
    playback_id: String,
    request: GuidedPlaybackRequest,
) -> Result<InteractiveLessonSessionDto, String> {
    state
        .guided_audio
        .confirm_completed(&playback_id, &request)?;
    state.guided_lessons.reference_completed(&request)
}

#[tauri::command]
fn cancel_guided_lesson_audio(state: State<'_, AppState>, playback_id: String) -> bool {
    state.guided_audio.cancel(&playback_id)
}

#[tauri::command]
async fn prepare_practice_audio(
    state: State<'_, AppState>,
    session_id: String,
    item_id: String,
) -> Result<GuidedAudioDto, String> {
    let text = state.practice.audio_text(&session_id, &item_id)?;
    state
        .guided_audio
        .prepare(
            state.paths.clone(),
            state.local_ai.clone(),
            GuidedPlaybackRequest {
                session_id,
                stage_id: "practice".into(),
                item_id,
            },
            GuidedPlaybackSource {
                text,
                asset_id: None,
                package_hash: "static-practice-v1".into(),
            },
        )
        .await
}

#[tauri::command]
fn get_static_tts_cache_status(state: State<'_, AppState>) -> StaticTtsCacheStatusDto {
    state.guided_audio.cache_status(&state.paths)
}

#[tauri::command]
fn clear_static_tts_cache(state: State<'_, AppState>) -> Result<StaticTtsCacheStatusDto, String> {
    state.guided_audio.clear_cache(&state.paths)
}

#[tauri::command]
fn select_guided_lesson_pronunciation_attempt(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: SelectGuidedAttemptRequest,
) -> Result<InteractiveLessonSessionDto, String> {
    let result = state.guided_lessons.select_pronunciation(&request)?;
    integrate_completed_guided(&app, &state, &request.session_id)?;
    Ok(result)
}

#[tauri::command]
fn submit_guided_lesson_exercise_attempt(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: SubmitExerciseAttemptRequest,
) -> Result<InteractiveLessonSessionDto, String> {
    let result = state.guided_lessons.submit_exercise(&request)?;
    integrate_completed_guided(&app, &state, &request.session_id)?;
    Ok(result)
}

#[tauri::command]
fn select_guided_lesson_exercise_attempt(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: SelectExerciseAttemptRequest,
) -> Result<InteractiveLessonSessionDto, String> {
    let result = state.guided_lessons.select_exercise(&request)?;
    integrate_completed_guided(&app, &state, &request.session_id)?;
    Ok(result)
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
async fn get_system_diagnostics(
    state: State<'_, AppState>,
) -> Result<SystemDiagnosticsDto, String> {
    Ok(system_diagnostics::run(&state.paths, &state.local_ai, &state.client).await)
}

#[tauri::command]
async fn export_diagnostic_report(state: State<'_, AppState>) -> Result<String, String> {
    let report = system_diagnostics::run(&state.paths, &state.local_ai, &state.client).await;
    system_diagnostics::sanitized_json(&report)
}

#[tauri::command]
fn get_welcome_state(state: State<'_, AppState>) -> Result<WelcomeStateDto, String> {
    ux_preferences::welcome_state(&state.paths.db_file())
}

#[tauri::command]
fn set_welcome_seen(state: State<'_, AppState>, seen: bool) -> Result<WelcomeStateDto, String> {
    ux_preferences::set_welcome_seen(&state.paths.db_file(), seen)
}

#[tauri::command]
fn create_app_backup(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<BackupSummaryDto, String> {
    let result = state.reliability.create_backup(&state.paths);
    match &result {
        Ok(backup) => {
            let _ = app.emit("english-ai-coach:backup-created", backup);
        }
        Err(_) => {
            let _ = reliability::record_event(
                &state.paths.db_file(),
                "error",
                "backup",
                "BACKUP_FAILED",
                None,
            );
        }
    }
    result
}

#[tauri::command]
fn list_app_backups(state: State<'_, AppState>) -> Vec<BackupSummaryDto> {
    reliability::list_backups(&state.paths)
}

#[tauri::command]
fn validate_app_backup(
    state: State<'_, AppState>,
    backup_id: String,
) -> Result<BackupValidationDto, String> {
    reliability::validate_backup(&state.paths, &backup_id)
}

fn restore_blocker(state: &AppState) -> Option<String> {
    let voice = state.voice_engine.get_state().ok()?;
    if matches!(
        voice.state,
        VoiceEngineState::Starting | VoiceEngineState::Running | VoiceEngineState::Stopping
    ) {
        return Some("End the active Voice Lesson before restoring data.".into());
    }
    if state.pronunciation_engine.is_analyzing() {
        return Some("Wait for Pronunciation analysis to finish before restoring data.".into());
    }
    None
}

#[tauri::command]
fn get_backup_status(state: State<'_, AppState>) -> BackupStatusDto {
    let blocker = restore_blocker(&state);
    state
        .reliability
        .status(&state.paths, blocker.is_none(), blocker)
}

#[tauri::command]
fn restore_app_backup(
    state: State<'_, AppState>,
    backup_id: String,
) -> Result<RestoreScheduledDto, String> {
    if let Some(reason) = restore_blocker(&state) {
        return Err(reason);
    }
    state.pronunciation_engine.shutdown();
    let result = state.reliability.schedule_restore(&state.paths, &backup_id);
    if result.is_err() {
        let _ = reliability::record_event(
            &state.paths.db_file(),
            "error",
            "restore",
            "RESTORE_FAILED",
            None,
        );
    }
    result
}

#[tauri::command]
fn get_backup_directory(state: State<'_, AppState>) -> String {
    state.paths.backups.display().to_string()
}

#[tauri::command]
fn open_backup_folder(state: State<'_, AppState>) -> Result<(), String> {
    #[cfg(windows)]
    {
        std::process::Command::new("explorer.exe")
            .arg(&state.paths.backups)
            .spawn()
            .map_err(|e| format!("Could not open backup folder: {e}"))?;
    }
    #[cfg(not(windows))]
    {
        let _ = state;
        return Err("Opening the backup folder is only supported on Windows.".into());
    }
    Ok(())
}

#[tauri::command]
fn list_recent_system_events(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<SystemEventDto>, String> {
    reliability::list_events(&state.paths.db_file(), limit)
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
    let streaming_enabled = match state.voice_performance.streaming_enabled() {
        Ok(value) => value,
        Err(error) => {
            log::error!("Could not read voice streaming setting; defaulting on: {error}");
            true
        }
    };
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
        streaming_enabled,
        &state.paths.temporary_audio,
        state.voice_performance.clone(),
        None,
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
    let summary = state.lessons.complete_lesson(&lesson.id)?;
    match state.gamification.sync() {
        Ok(result) => {
            let _ = app.emit("english-ai-coach:gamification-changed", &result);
        }
        Err(error) => log::error!("Gamification lesson sync failed: {error}"),
    }
    Ok(summary)
}

#[tauri::command]
fn cancel_current_teacher_response(state: State<'_, AppState>) -> Result<bool, String> {
    state.voice_engine.cancel_current_response()
}

#[tauri::command]
fn get_streaming_voice_response_enabled(state: State<'_, AppState>) -> Result<bool, String> {
    state.voice_performance.streaming_enabled()
}

#[tauri::command]
fn set_streaming_voice_response_enabled(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<bool, String> {
    state.voice_performance.set_streaming_enabled(enabled)
}

#[tauri::command]
fn get_gamification_overview(
    state: State<'_, AppState>,
) -> Result<GamificationOverviewDto, String> {
    state.gamification.overview()
}

#[tauri::command]
fn get_gamification_profile(state: State<'_, AppState>) -> Result<GamificationProfileDto, String> {
    state.gamification.profile()
}

#[tauri::command]
fn update_weekly_practice_goal(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    minutes: u32,
) -> Result<GamificationProfileDto, String> {
    let profile = state.gamification.update_weekly_goal(minutes)?;
    let _ = app.emit("english-ai-coach:gamification-changed", ());
    Ok(profile)
}

#[tauri::command]
fn list_achievements(state: State<'_, AppState>) -> Result<Vec<AchievementDto>, String> {
    state.gamification.sync()?;
    state.gamification.achievements()
}

#[tauri::command]
fn sync_gamification(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<GamificationSyncResult, String> {
    let result = state.gamification.sync()?;
    let _ = app.emit("english-ai-coach:gamification-changed", &result);
    Ok(result)
}

#[tauri::command]
fn get_review_overview(state: State<'_, AppState>) -> Result<ReviewOverviewDto, String> {
    state.review.overview()
}

#[tauri::command]
fn preview_review_queue(
    state: State<'_, AppState>,
    mode: review::ReviewMode,
    item_count: u32,
) -> Result<ReviewQueuePreviewDto, String> {
    state.review.preview(mode, item_count)
}

#[tauri::command]
fn start_review_session(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: StartReviewSessionRequest,
) -> Result<ReviewSessionDto, String> {
    let result = state.review.start(request)?;
    let _ = app.emit("english-ai-coach:review-changed", ());
    Ok(result)
}

#[tauri::command]
fn resume_review_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<ReviewSessionDto, String> {
    state.review.resume(&session_id)
}

#[tauri::command]
fn get_review_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Option<ReviewSessionDto>, String> {
    state.review.get(&session_id)
}

#[tauri::command]
fn abandon_review_session(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    session_id: String,
) -> Result<ReviewSessionDto, String> {
    let result = state.review.abandon(&session_id)?;
    let _ = app.emit("english-ai-coach:review-changed", ());
    Ok(result)
}

#[tauri::command]
fn submit_review_item(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: SubmitReviewItemRequest,
) -> Result<ReviewSubmitResult, String> {
    let result = state.review.submit(request)?;
    if result.vocabulary_status_changed {
        if let Err(error) = state.summaries.refresh_summary() {
            log::error!("Review saved, but learning summary refresh failed: {error}");
        }
        let _ = app.emit("english-ai-coach:learning-data-changed", ());
    }
    let _ = app.emit("english-ai-coach:review-changed", ());
    Ok(result)
}

#[tauri::command]
fn list_recent_review_sessions(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<ReviewSessionSummaryDto>, String> {
    state.review.list_recent(limit)
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
    speech::transcribe_official_local_ai(state.paths.clone(), state.local_ai.clone(), audio_base64)
        .await
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
async fn get_pronunciation_engine_status(
    state: State<'_, AppState>,
) -> Result<PronunciationEngineStatus, String> {
    let manager = state.pronunciation_engine.clone();
    let local_ai = state.local_ai.clone();
    tokio::task::spawn_blocking(move || manager.status(&local_ai, true))
        .await
        .map_err(|error| format!("Pronunciation readiness task failed: {error}"))
}

#[tauri::command]
async fn analyze_pronunciation(
    state: State<'_, AppState>,
    request: AnalyzePronunciationRequest,
) -> Result<PronunciationAttemptDto, String> {
    validate_pronunciation_target(&request.target_text)?;
    if !matches!(request.source_type.as_str(), "custom" | "vocabulary") {
        return Err("Invalid pronunciation source type.".into());
    }
    if request.source_type == "vocabulary"
        && request.source_id.as_deref().map_or(true, str::is_empty)
    {
        return Err("Vocabulary pronunciation practice requires a source id.".into());
    }
    analyze_pronunciation_audio(
        state.paths.clone(),
        state.local_ai.clone(),
        state.pronunciation_engine.clone(),
        state.pronunciation.clone(),
        request.target_text,
        request.audio_base64,
        request.source_type,
        request.source_id,
    )
    .await
}

async fn analyze_pronunciation_audio(
    paths: LocalPaths,
    local_ai: LocalAiPaths,
    manager: PronunciationEngineManager,
    repository: PronunciationRepository,
    target_text: String,
    audio_base64: String,
    source_type: String,
    source_id: Option<String>,
) -> Result<PronunciationAttemptDto, String> {
    validate_pronunciation_target(&target_text)?;
    let audio = STANDARD
        .decode(&audio_base64)
        .map_err(|_| "Pronunciation audio payload is invalid.".to_owned())?;
    if audio.len() < 48 || audio.len() > 1_000_000 {
        return Err("Pronunciation recording is empty or too long.".into());
    }
    let request_id = uuid::Uuid::new_v4().to_string();
    let audio_path = paths
        .temporary_audio
        .join(format!("pronunciation-{request_id}.wav"));
    std::fs::write(&audio_path, audio)
        .map_err(|error| format!("Could not create temporary pronunciation audio: {error}"))?;
    let heard = pronunciation_speech::transcribe_path(
        local_ai.clone(),
        &audio_path,
        &paths.temporary_audio,
    )
    .await;
    let result = match heard {
        Ok((heard, _)) => {
            let target = target_text.clone();
            let path = audio_path.clone();
            let id = request_id.clone();
            tokio::task::spawn_blocking(move || {
                manager.analyze(&local_ai, &id, &target, &heard, &path)
            })
            .await
            .map_err(|error| format!("Pronunciation analysis task failed: {error}"))?
        }
        Err(error) => Err(error),
    };
    let _ = std::fs::remove_file(&audio_path);
    let result = result?;
    validate_result(&result, &target_text)?;
    repository.save(&result, &source_type, source_id.as_deref())
}

#[tauri::command]
async fn submit_guided_lesson_pronunciation(
    state: State<'_, AppState>,
    request: GuidedPronunciationRequest,
) -> Result<InteractiveLessonSessionDto, String> {
    let decoded = STANDARD
        .decode(&request.audio_base64)
        .map_err(|_| "Pronunciation audio payload is invalid.".to_owned())?;
    if decoded.len() < 48 || decoded.len() > 1_000_000 {
        return Err("Pronunciation recording is empty or too long.".into());
    }
    let context = state.guided_lessons.begin_pronunciation(&request)?;
    let result = analyze_pronunciation_audio(
        state.paths.clone(),
        state.local_ai.clone(),
        state.pronunciation_engine.clone(),
        state.pronunciation.clone(),
        context.target_text,
        request.audio_base64,
        "interactive_lesson".into(),
        Some(context.attempt_id.clone()),
    )
    .await;
    match result {
        Ok(attempt) => state.guided_lessons.finish_pronunciation(
            &context.attempt_id,
            &attempt.status,
            Some(&attempt.id),
            None,
        ),
        Err(error) => {
            let _ = state.guided_lessons.finish_pronunciation(
                &context.attempt_id,
                "failed",
                None,
                Some("analysis_failed"),
            );
            Err(error)
        }
    }
}

#[tauri::command]
fn cancel_guided_lesson_pronunciation(state: State<'_, AppState>) -> bool {
    state.pronunciation_engine.cancel();
    true
}

#[tauri::command]
fn cancel_pronunciation_analysis(state: State<'_, AppState>) -> Result<bool, String> {
    state.pronunciation_engine.cancel();
    Ok(true)
}

#[tauri::command]
fn list_pronunciation_attempts(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<PronunciationAttemptDto>, String> {
    state.pronunciation.list(limit)
}

#[tauri::command]
fn get_pronunciation_attempt(
    state: State<'_, AppState>,
    attempt_id: String,
) -> Result<Option<PronunciationAttemptDto>, String> {
    state.pronunciation.get(&attempt_id)
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
    app: tauri::AppHandle,
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
    let result = state.placement.finalize(&attempt_id, evaluation)?;
    match state.gamification.sync() {
        Ok(sync) => {
            let _ = app.emit("english-ai-coach:gamification-changed", &sync);
        }
        Err(error) => log::error!("Gamification placement sync failed: {error}"),
    }
    Ok(result)
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
            let restore_result = reliability::process_pending_restore(&paths).map_err(std::io::Error::other)?;
            database::migrate(&paths.db_file()).map_err(std::io::Error::other)?;
            let lesson_repository = LessonRepository::new(paths.db_file());
            let recovered = lesson_repository
                .recover_stale_lessons()
                .map_err(std::io::Error::other)?;
            if recovered > 0 {
                log::info!("Recovered {recovered} unfinished lesson(s) as interrupted.");
                let _ = reliability::record_event(&paths.db_file(), "recovery", "lesson", "STALE_SESSION_RECOVERED", Some(serde_json::json!({"recoveredCount":recovered})));
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
            let cleanup = reliability::cleanup_owned_temp(&paths);
            if cleanup.failed > 0 {
                let _ = reliability::record_event(&paths.db_file(), "warning", "temporary_files", "TEMP_CLEANUP_FAILED", Some(serde_json::json!({"failedCount":cleanup.failed})));
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
            let guided_learning = GuidedLearningIntegrationRepository::new(paths.db_file());
            match guided_learning.sync_all_completed() {
                Ok(result) if result.integrated_sessions > 0 => {
                    log::info!(
                        "Integrated {} completed Guided Lesson session(s).",
                        result.integrated_sessions
                    );
                    if let Err(error) = summaries.refresh_summary() {
                        log::error!("Guided learning summary startup refresh failed: {error}");
                    }
                }
                Ok(_) => {}
                Err(error) => log::error!("Guided learning startup sync failed: {error}"),
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
            let guided_content_root = if cfg!(debug_assertions) {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/interactive-lessons")
            } else {
                app.path().resource_dir().map_err(|error| std::io::Error::other(format!("Resource directory unavailable: {error}")))?.join("interactive-lessons")
            };
            let guided_registry = InteractiveLessonContentRegistry::load(guided_content_root);
            let curriculum_content_root = if cfg!(debug_assertions) {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/curriculum")
            } else {
                app.path().resource_dir().map_err(|error| std::io::Error::other(format!("Resource directory unavailable: {error}")))?.join("curriculum")
            };
            let curriculum = CurriculumService::new(
                CurriculumRegistry::load(curriculum_content_root, &guided_registry),
                paths.db_file(),
                profiles.clone(),
            );
            let toeic_content_root = if cfg!(debug_assertions) {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/toeic/item-bank-v1")
            } else {
                app.path().resource_dir().map_err(|error| std::io::Error::other(format!("Resource directory unavailable: {error}")))?.join("toeic/item-bank-v1")
            };
            let toeic_bank = ToeicItemBank::load(toeic_content_root).map_err(std::io::Error::other)?;
            let toeic = ToeicRepository::new(paths.db_file(), toeic_bank).map_err(std::io::Error::other)?;
            let guided_lessons = InteractiveLessonEngine::new(
                guided_registry,
                InteractiveLessonRepository::new(paths.db_file()),
                profiles.clone(),
                paths.interactive_lesson_assets.clone(),
            );
            let recovered_guided_attempts=guided_lessons.recover_interrupted_attempts().map_err(std::io::Error::other)?;
            if recovered_guided_attempts>0 { log::info!("Recovered {recovered_guided_attempts} interrupted Guided Lesson pronunciation attempt(s)."); }
            let placement_evaluator =
                PlacementSpeakingEvaluator::new().map_err(std::io::Error::other)?;
            let gamification = GamificationRepository::new(paths.db_file());
            if let Err(error) = gamification.sync() {
                log::error!("Gamification startup sync failed: {error}");
            }
            let review = ReviewRepository::new(paths.db_file());
            let practice = PracticeRepository::new(paths.db_file());
            let voice_performance = VoicePerformanceRepository::new(paths.db_file());
            let pronunciation = PronunciationRepository::new(paths.db_file());
            let guided_conversations = GuidedConversationRepository::new(&paths.db_file());
            let guided_analysis = InteractiveLessonAnalysisService::new(paths.db_file())
                .map_err(std::io::Error::other)?;
            let recovered_guided_analyses = guided_analysis
                .recover_stale()
                .map_err(std::io::Error::other)?;
            if recovered_guided_analyses > 0 {
                log::info!(
                    "Recovered {recovered_guided_analyses} interrupted Guided Lesson analysis request(s)."
                );
            }
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
                gamification,
                review,
                practice,
                voice_performance,
                pronunciation_engine: PronunciationEngineManager::default(),
                pronunciation,
                reliability: ReliabilityManager::default(),
                guided_lessons,
                guided_audio: GuidedLessonAudioRuntime::default(),
                guided_conversations,
                guided_analysis,
                guided_learning,
                curriculum,
                toeic,
            });
            if let Some(result) = restore_result { let _ = app.emit("english-ai-coach:data-restored", result); }
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                let state = window.state::<AppState>();
                state.voice_engine.shutdown();
                state.pronunciation_engine.shutdown();
                state.guided_audio.shutdown();
                if let Err(error) = state.lessons.interrupt_active() {
                    log::error!("Could not preserve interrupted lesson on shutdown: {error}");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_toeic_overview,
            start_toeic_session,
            get_toeic_session,
            submit_toeic_answer,
            advance_toeic_session,
            abandon_toeic_session,
            record_toeic_active_time,
            get_toeic_result,
            get_toeic_review,
            list_toeic_history,
            prepare_toeic_audio,
            complete_toeic_audio,
            cancel_toeic_audio,
            diagnostics,
            probe_local_voice_engine,
            get_system_diagnostics,
            export_diagnostic_report,
            get_welcome_state,
            set_welcome_seen,
            get_guided_lesson_overview,
            get_course_catalog,
            list_guided_lessons,
            get_guided_lesson,
            get_active_guided_lesson_session,
            start_guided_lesson,
            resume_guided_lesson,
            get_guided_lesson_session,
            start_guided_conversation,
            stop_guided_conversation,
            finish_guided_conversation,
            complete_guided_lesson_stage,
            skip_guided_lesson_stage,
            abandon_guided_lesson,
            list_recent_guided_lesson_sessions,
            get_guided_lesson_analysis,
            analyze_guided_lesson,
            retry_guided_lesson_conversation_analysis,
            finalize_guided_lesson_analysis,
            record_guided_practice_time,
            prepare_guided_lesson_audio,
            complete_guided_lesson_audio,
            cancel_guided_lesson_audio,
            prepare_practice_audio,
            get_static_tts_cache_status,
            clear_static_tts_cache,
            submit_guided_lesson_pronunciation,
            cancel_guided_lesson_pronunciation,
            select_guided_lesson_pronunciation_attempt,
            submit_guided_lesson_exercise_attempt,
            select_guided_lesson_exercise_attempt,
            create_app_backup,
            list_app_backups,
            validate_app_backup,
            restore_app_backup,
            get_backup_status,
            get_backup_directory,
            open_backup_folder,
            list_recent_system_events,
            get_lesson_modes,
            start_lesson,
            end_lesson,
            cancel_current_teacher_response,
            get_streaming_voice_response_enabled,
            set_streaming_voice_response_enabled,
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
            get_pronunciation_engine_status,
            analyze_pronunciation,
            cancel_pronunciation_analysis,
            list_pronunciation_attempts,
            get_pronunciation_attempt,
            get_student_learning_profile,
            update_student_learning_profile,
            get_student_profile_context_status,
            get_gamification_overview,
            get_gamification_profile,
            update_weekly_practice_goal,
            list_achievements,
            sync_gamification,
            get_review_overview,
            get_practice_availability,
            start_practice_session,
            get_practice_session,
            record_practice_time,
            complete_practice_item,
            complete_practice_session,
            abandon_practice_session,
            preview_review_queue,
            start_review_session,
            resume_review_session,
            get_review_session,
            abandon_review_session,
            submit_review_item,
            list_recent_review_sessions,
            list_ollama_models,
            transcribe_audio,
            chat_teacher,
            synthesize_speech
        ])
        .run(tauri::generate_context!())
        .expect("error while running English AI Coach");
}
