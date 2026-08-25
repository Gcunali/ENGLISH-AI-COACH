import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Achievement, CorrectionCandidate, DashboardSummary, Diagnostics, GamificationOverview, GamificationProfile, GamificationSyncResult, LearningMemorySummary, LearningMemorySyncResult, Lesson, LessonAnalysis, LessonDetails, LessonHistoryFilter, LessonHistoryPage, LessonModeDefinition, LessonStartRequest, LessonSummary, LocalVoiceEngineProbe, OllamaModel, PlacementAttempt, PlacementOverview, PlacementResult, PlacementSession, ProgressOverview, RecurringMistake, RecurringMistakeDetails, ReviewMode, ReviewOutcome, ReviewOverview, ReviewQueuePreview, ReviewSession, ReviewSessionSummary, ReviewSubmitResult, StartLessonResult, StartReviewSessionRequest, StudentLearningProfile, StudentLearningSummary, StudentProfileContextStatus, TranscriptMessage, UpdateStudentProfileRequest, VocabularyFilter, VocabularyItem, VocabularyItemDetails, VocabularyPage, VocabularySort, VocabularyStatus, VocabularySummary, VoiceEngineEvent, VoiceEngineStatus } from '../types'
import type { PronunciationAttempt, PronunciationEngineStatus } from '../types'
import type { BackupStatus, BackupSummary, BackupValidation, RestoreScheduled, SystemDiagnostics, SystemEvent } from '../types'

function isTauri(): boolean {
  return '__TAURI_INTERNALS__' in window
}

export async function getDiagnostics(): Promise<Diagnostics> {
  if (!isTauri()) {
    return {
      components: [
        { name: 'ollama', label: 'Ollama', ready: false, detail: 'Desktop runtime required' },
        { name: 'llm', label: 'Language model', ready: false, detail: 'qwen3.5:4b not detected' },
        { name: 'whisper', label: 'Whisper', ready: false, detail: 'Desktop runtime required' },
        { name: 'vad', label: 'Voice activity detection', ready: true, detail: 'RMS VAD configured; Silero is optional' },
        { name: 'piper', label: 'Piper voice', ready: false, detail: 'Desktop runtime required' },
        { name: 'microphone', label: 'Microphone', ready: false, detail: 'Not tested yet' },
        { name: 'speaker', label: 'Speaker', ready: false, detail: 'Not tested yet' },
        { name: 'sqlite', label: 'Local database', ready: false, detail: 'Desktop runtime required' },
      ],
      dataDirectory: 'Desktop runtime only', offlineReady: false, platform: navigator.platform,
    }
  }
  return invoke<Diagnostics>('diagnostics')
}

export async function probeLocalVoiceEngine(): Promise<LocalVoiceEngineProbe> {
  if (!isTauri()) {
    return {
      projectRoot: 'Desktop runtime only',
      localAiRoot: 'Desktop runtime only',
      whisper: {
        cliFound: false,
        cliPath: '',
        streamFound: false,
        streamPath: '',
        modelFound: false,
        modelPath: '',
        modelName: 'ggml-small.en-q5_1.bin',
        threads: 12,
        additionalModels: [],
      },
      ollama: {
        reachable: false,
        baseUrl: 'http://127.0.0.1:11434',
        modelFound: false,
        modelName: 'qwen3.5:4b',
      },
      piper: {
        pythonFound: false,
        pythonPath: '',
        installed: false,
        version: null,
        voiceFound: false,
        voiceConfigFound: false,
        voiceModelPath: '',
        voiceConfigPath: '',
        voiceName: 'en_US-lessac-medium',
      },
      voiceDefaults: {
        whisperModel: 'ggml-small.en-q5_1.bin',
        whisperThreads: 12,
        silenceToStopSeconds: 3.5,
        preRollSeconds: 0.4,
        startVoiceBlocks: 3,
        minimumVoiceThreshold: 350,
        noiseMultiplier: 3,
        piperVoice: 'en_US-lessac-medium',
        ttsStartSilenceSeconds: 0.5,
        ollamaModel: 'qwen3.5:4b',
        ollamaThinking: false,
      },
      optionalComponents: { sileroFound: false, sileroPath: '' },
      offlineReady: false,
      problems: ['Local voice engine probe requires the desktop runtime.'],
    }
  }
  return invoke<LocalVoiceEngineProbe>('probe_local_voice_engine')
}

export async function getLessonModes(): Promise<LessonModeDefinition[]> {
  if (!isTauri()) throw new Error('Lesson modes require the desktop runtime.')
  return invoke<LessonModeDefinition[]>('get_lesson_modes')
}

export async function startLesson(request: LessonStartRequest): Promise<StartLessonResult> {
  if (!isTauri()) throw new Error('The local voice engine requires the desktop runtime.')
  return invoke<StartLessonResult>('start_lesson', { request })
}

export async function endLesson(): Promise<LessonSummary> {
  if (!isTauri()) throw new Error('Lesson persistence requires the desktop runtime.')
  return invoke<LessonSummary>('end_lesson')
}

export async function getVoiceEngineState(): Promise<VoiceEngineStatus> {
  if (!isTauri()) return { state: 'stopped', processId: null }
  return invoke<VoiceEngineStatus>('get_voice_engine_state')
}

export async function cancelCurrentTeacherResponse(): Promise<boolean> {
  if (!isTauri()) return false
  return invoke<boolean>('cancel_current_teacher_response')
}

export async function getStreamingVoiceResponseEnabled(): Promise<boolean> {
  if (!isTauri()) throw new Error('Voice performance settings require the desktop runtime.')
  return invoke<boolean>('get_streaming_voice_response_enabled')
}

export async function setStreamingVoiceResponseEnabled(enabled: boolean): Promise<boolean> {
  if (!isTauri()) throw new Error('Voice performance settings require the desktop runtime.')
  return invoke<boolean>('set_streaming_voice_response_enabled', { enabled })
}

export async function getActiveLesson(): Promise<Lesson | null> {
  if (!isTauri()) return null
  return invoke<Lesson | null>('get_active_lesson')
}

export async function getLesson(lessonId: string): Promise<Lesson | null> {
  if (!isTauri()) return null
  return invoke<Lesson | null>('get_lesson', { lessonId })
}

export async function getLatestCompletedLesson(): Promise<Lesson | null> {
  if (!isTauri()) return null
  return invoke<Lesson | null>('get_latest_completed_lesson')
}

export async function getLessonTranscript(lessonId: string): Promise<TranscriptMessage[]> {
  if (!isTauri()) return []
  return invoke<TranscriptMessage[]>('get_lesson_transcript', { lessonId })
}

export async function getLessonCorrections(lessonId: string): Promise<CorrectionCandidate[]> {
  if (!isTauri()) return []
  return invoke<CorrectionCandidate[]>('get_lesson_corrections', { lessonId })
}

export async function getLessonAnalysis(lessonId: string): Promise<LessonAnalysis | null> {
  if (!isTauri()) return null
  return invoke<LessonAnalysis | null>('get_lesson_analysis', { lessonId })
}

export async function analyzeLesson(lessonId: string): Promise<LessonAnalysis> {
  if (!isTauri()) throw new Error('Lesson analysis requires the desktop runtime.')
  return invoke<LessonAnalysis>('analyze_lesson', { lessonId })
}

export async function retryLessonAnalysis(lessonId: string): Promise<LessonAnalysis> {
  if (!isTauri()) throw new Error('Lesson analysis requires the desktop runtime.')
  return invoke<LessonAnalysis>('retry_lesson_analysis', { lessonId })
}

export async function getDashboardSummary(): Promise<DashboardSummary> {
  if (!isTauri()) throw new Error('Dashboard data requires the desktop runtime.')
  return invoke<DashboardSummary>('get_dashboard_summary')
}

export async function listLessons(
  filter: LessonHistoryFilter,
  limit: number,
  offset: number,
): Promise<LessonHistoryPage> {
  if (!isTauri()) throw new Error('Lesson history requires the desktop runtime.')
  return invoke<LessonHistoryPage>('list_lessons', { filter, limit, offset })
}

export async function getLessonDetails(lessonId: string): Promise<LessonDetails | null> {
  if (!isTauri()) throw new Error('Lesson details require the desktop runtime.')
  return invoke<LessonDetails | null>('get_lesson_details', { lessonId })
}

export async function getProgressOverview(): Promise<ProgressOverview> {
  if (!isTauri()) throw new Error('Progress data requires the desktop runtime.')
  return invoke<ProgressOverview>('get_progress_overview')
}

export async function getVocabularySummary(): Promise<VocabularySummary> {
  if (!isTauri()) throw new Error('Vocabulary data requires the desktop runtime.')
  return invoke<VocabularySummary>('get_vocabulary_summary')
}

export async function getLearningMemorySummary(): Promise<LearningMemorySummary> {
  if (!isTauri()) throw new Error('Learning memory requires the desktop runtime.')
  return invoke<LearningMemorySummary>('get_learning_memory_summary')
}

export async function listVocabularyItems(
  filter: VocabularyFilter,
  search: string,
  sort: VocabularySort,
  limit: number,
  offset: number,
): Promise<VocabularyPage> {
  if (!isTauri()) throw new Error('Vocabulary data requires the desktop runtime.')
  return invoke<VocabularyPage>('list_vocabulary_items', { filter, search, sort, limit, offset })
}

export async function getVocabularyItem(vocabularyId: string): Promise<VocabularyItemDetails | null> {
  if (!isTauri()) throw new Error('Vocabulary data requires the desktop runtime.')
  return invoke<VocabularyItemDetails | null>('get_vocabulary_item', { vocabularyId })
}

export async function updateVocabularyStatus(vocabularyId: string, status: VocabularyStatus): Promise<VocabularyItem> {
  if (!isTauri()) throw new Error('Vocabulary data requires the desktop runtime.')
  return invoke<VocabularyItem>('update_vocabulary_status', { vocabularyId, status })
}

export async function listRecurringMistakes(limit: number): Promise<RecurringMistake[]> {
  if (!isTauri()) throw new Error('Recurring mistakes require the desktop runtime.')
  return invoke<RecurringMistake[]>('list_recurring_mistakes', { limit })
}

export async function getRecurringMistake(mistakeId: string): Promise<RecurringMistakeDetails | null> {
  if (!isTauri()) throw new Error('Recurring mistakes require the desktop runtime.')
  return invoke<RecurringMistakeDetails | null>('get_recurring_mistake', { mistakeId })
}

export async function syncLearningMemory(): Promise<LearningMemorySyncResult> {
  if (!isTauri()) throw new Error('Learning memory sync requires the desktop runtime.')
  return invoke<LearningMemorySyncResult>('sync_learning_memory')
}

export async function getStudentLearningSummary(): Promise<StudentLearningSummary> {
  if (!isTauri()) throw new Error('Learning summary requires the desktop runtime.')
  return invoke<StudentLearningSummary>('get_student_learning_summary')
}

export async function getLearningMemoryEnabled(): Promise<boolean> {
  if (!isTauri()) throw new Error('Learning memory settings require the desktop runtime.')
  return invoke<boolean>('get_learning_memory_enabled')
}

export async function setLearningMemoryEnabled(enabled: boolean): Promise<boolean> {
  if (!isTauri()) throw new Error('Learning memory settings require the desktop runtime.')
  return invoke<boolean>('set_learning_memory_enabled', { enabled })
}

export async function subscribeVoiceEngineEvents(
  handler: (event: VoiceEngineEvent) => void,
): Promise<UnlistenFn> {
  if (!isTauri()) return () => undefined
  return listen<VoiceEngineEvent>('voice-engine-event', (event) => handler(event.payload))
}

export async function transcribeAudio(audioBase64: string): Promise<{ text: string; elapsedMs: number }> {
  if (!isTauri()) throw new Error('Local speech recognition is available in the desktop app.')
  return invoke('transcribe_audio', { audioBase64 })
}

export async function chatTeacher(text: string, model: string): Promise<{ text: string; elapsedMs: number }> {
  if (!isTauri()) throw new Error('Local AI is available in the desktop app.')
  return invoke('chat_teacher', { text, model })
}

export async function synthesizeSpeech(text: string): Promise<{ audioBase64: string; mimeType: string; elapsedMs: number }> {
  if (!isTauri()) throw new Error('Local speech synthesis is available in the desktop app.')
  return invoke('synthesize_speech', { text })
}

export async function listOllamaModels(): Promise<OllamaModel[]> {
  if (!isTauri()) return []
  return invoke<OllamaModel[]>('list_ollama_models')
}

export async function getPlacementOverview(): Promise<PlacementOverview> {
  if (!isTauri()) throw new Error('Placement Test requires the desktop runtime.')
  return invoke<PlacementOverview>('get_placement_overview')
}
export async function startPlacementTest(startOver = false): Promise<PlacementSession> {
  if (!isTauri()) throw new Error('Placement Test requires the desktop runtime.')
  return invoke<PlacementSession>('start_placement_test', { startOver })
}
export async function resumePlacementTest(attemptId: string): Promise<PlacementSession> {
  if (!isTauri()) throw new Error('Placement Test requires the desktop runtime.')
  return invoke<PlacementSession>('resume_placement_test', { attemptId })
}
export async function abandonPlacementTest(attemptId: string): Promise<PlacementAttempt> {
  if (!isTauri()) throw new Error('Placement Test requires the desktop runtime.')
  return invoke<PlacementAttempt>('abandon_placement_test', { attemptId })
}
export async function submitPlacementAnswer(attemptId: string, questionId: string, selectedOptionId: string): Promise<PlacementSession> {
  if (!isTauri()) throw new Error('Placement Test requires the desktop runtime.')
  return invoke<PlacementSession>('submit_placement_answer', { request: { attemptId, questionId, selectedOptionId } })
}
export async function capturePlacementSpeakingResponse(audioBase64: string): Promise<{ text: string; elapsedMs: number }> {
  if (!isTauri()) throw new Error('Placement speech capture requires the desktop runtime.')
  return invoke('capture_placement_speaking_response', { audioBase64 })
}
export async function confirmPlacementSpeakingResponse(attemptId: string, promptId: string, transcript: string): Promise<PlacementSession> {
  if (!isTauri()) throw new Error('Placement Test requires the desktop runtime.')
  return invoke<PlacementSession>('confirm_placement_speaking_response', { request: { attemptId, promptId, transcript } })
}
export async function skipPlacementSpeaking(attemptId: string): Promise<PlacementSession> {
  if (!isTauri()) throw new Error('Placement Test requires the desktop runtime.')
  return invoke<PlacementSession>('skip_placement_speaking', { attemptId })
}
export async function finalizePlacementTest(attemptId: string): Promise<PlacementResult> {
  if (!isTauri()) throw new Error('Placement Test requires the desktop runtime.')
  return invoke<PlacementResult>('finalize_placement_test', { attemptId })
}
export async function getPlacementResult(attemptId: string): Promise<PlacementResult | null> {
  if (!isTauri()) throw new Error('Placement Test requires the desktop runtime.')
  return invoke<PlacementResult | null>('get_placement_result', { attemptId })
}
export async function getCurrentPlacementResult(): Promise<PlacementResult | null> {
  if (!isTauri()) throw new Error('Placement Test requires the desktop runtime.')
  return invoke<PlacementResult | null>('get_current_placement_result')
}
export async function listPlacementAttempts(): Promise<PlacementAttempt[]> {
  if (!isTauri()) throw new Error('Placement Test requires the desktop runtime.')
  return invoke<PlacementAttempt[]>('list_placement_attempts')
}

export async function getPronunciationEngineStatus():Promise<PronunciationEngineStatus>{if(!isTauri())return{installed:false,available:false,ready:false,engineVersion:1,scoreVersion:1,resultSchemaVersion:1,modelId:'facebook/wav2vec2-lv-60-espeak-cv-ft',modelRevision:'',phonemizerReady:false,loadMs:null,lastError:'Desktop runtime required.'};return invoke<PronunciationEngineStatus>('get_pronunciation_engine_status')}
export async function analyzePronunciation(targetText:string,audioBase64:string,sourceType:'custom'|'vocabulary'='custom',sourceId:string|null=null):Promise<PronunciationAttempt>{if(!isTauri())throw new Error('Pronunciation Practice requires the desktop runtime.');return invoke<PronunciationAttempt>('analyze_pronunciation',{request:{targetText,audioBase64,sourceType,sourceId}})}
export async function cancelPronunciationAnalysis():Promise<boolean>{if(!isTauri())return false;return invoke<boolean>('cancel_pronunciation_analysis')}
export async function listPronunciationAttempts(limit=20):Promise<PronunciationAttempt[]>{if(!isTauri())return[];return invoke<PronunciationAttempt[]>('list_pronunciation_attempts',{limit})}
export async function getPronunciationAttempt(attemptId:string):Promise<PronunciationAttempt|null>{if(!isTauri())return null;return invoke<PronunciationAttempt|null>('get_pronunciation_attempt',{attemptId})}

export async function getSystemDiagnostics():Promise<SystemDiagnostics>{if(!isTauri())throw new Error('System Diagnostics requires the desktop runtime.');return invoke<SystemDiagnostics>('get_system_diagnostics')}
export async function exportDiagnosticReport():Promise<string>{if(!isTauri())throw new Error('Diagnostic export requires the desktop runtime.');return invoke<string>('export_diagnostic_report')}
export async function listRecentSystemEvents(limit=20):Promise<SystemEvent[]>{if(!isTauri())return[];return invoke<SystemEvent[]>('list_recent_system_events',{limit})}
export async function createAppBackup():Promise<BackupSummary>{if(!isTauri())throw new Error('Backup requires the desktop runtime.');return invoke<BackupSummary>('create_app_backup')}
export async function listAppBackups():Promise<BackupSummary[]>{if(!isTauri())return[];return invoke<BackupSummary[]>('list_app_backups')}
export async function validateAppBackup(backupId:string):Promise<BackupValidation>{if(!isTauri())throw new Error('Backup validation requires the desktop runtime.');return invoke<BackupValidation>('validate_app_backup',{backupId})}
export async function restoreAppBackup(backupId:string):Promise<RestoreScheduled>{if(!isTauri())throw new Error('Restore requires the desktop runtime.');return invoke<RestoreScheduled>('restore_app_backup',{backupId})}
export async function getBackupStatus():Promise<BackupStatus>{if(!isTauri())return{operation:'idle',error:null,backupDirectory:'Desktop runtime only',lastBackup:null,restoreAllowed:false,restoreBlockReason:'Desktop runtime required.',pendingRestart:false,lastRestore:null};return invoke<BackupStatus>('get_backup_status')}
export async function getBackupDirectory():Promise<string>{if(!isTauri())return'Desktop runtime only';return invoke<string>('get_backup_directory')}
export async function openBackupFolder():Promise<void>{if(!isTauri())throw new Error('Opening the backup folder requires the desktop runtime.');await invoke('open_backup_folder')}

export async function getStudentLearningProfile(): Promise<StudentLearningProfile> {
  if (!isTauri()) throw new Error('Student Profile requires the desktop runtime.')
  return invoke<StudentLearningProfile>('get_student_learning_profile')
}
export async function updateStudentLearningProfile(request: UpdateStudentProfileRequest): Promise<StudentLearningProfile> {
  if (!isTauri()) throw new Error('Student Profile requires the desktop runtime.')
  return invoke<StudentLearningProfile>('update_student_learning_profile', { request })
}
export async function getStudentProfileContextStatus(): Promise<StudentProfileContextStatus> {
  if (!isTauri()) throw new Error('Student Profile requires the desktop runtime.')
  return invoke<StudentProfileContextStatus>('get_student_profile_context_status')
}

export async function getGamificationOverview(): Promise<GamificationOverview> {
  if (!isTauri()) throw new Error('Practice data requires the desktop runtime.')
  return invoke<GamificationOverview>('get_gamification_overview')
}
export async function getGamificationProfile(): Promise<GamificationProfile> {
  if (!isTauri()) throw new Error('Practice settings require the desktop runtime.')
  return invoke<GamificationProfile>('get_gamification_profile')
}
export async function updateWeeklyPracticeGoal(minutes: number): Promise<GamificationProfile> {
  if (!isTauri()) throw new Error('Practice settings require the desktop runtime.')
  return invoke<GamificationProfile>('update_weekly_practice_goal', { minutes })
}
export async function listAchievements(): Promise<Achievement[]> {
  if (!isTauri()) throw new Error('Achievements require the desktop runtime.')
  return invoke<Achievement[]>('list_achievements')
}
export async function syncGamification(): Promise<GamificationSyncResult> {
  if (!isTauri()) throw new Error('Practice sync requires the desktop runtime.')
  return invoke<GamificationSyncResult>('sync_gamification')
}
export async function subscribeGamificationChanges(handler: (result: GamificationSyncResult | null) => void): Promise<UnlistenFn> {
  if (!isTauri()) return () => undefined
  return listen<GamificationSyncResult | null>('english-ai-coach:gamification-changed', (event) => handler(event.payload))
}

export async function getReviewOverview(): Promise<ReviewOverview> { if(!isTauri()) throw new Error('Review requires the desktop runtime.'); return invoke<ReviewOverview>('get_review_overview') }
export async function previewReviewQueue(mode:ReviewMode,itemCount:number):Promise<ReviewQueuePreview>{if(!isTauri())throw new Error('Review requires the desktop runtime.');return invoke<ReviewQueuePreview>('preview_review_queue',{mode,itemCount})}
export async function startReviewSession(request:StartReviewSessionRequest):Promise<ReviewSession>{if(!isTauri())throw new Error('Review requires the desktop runtime.');return invoke<ReviewSession>('start_review_session',{request})}
export async function resumeReviewSession(sessionId:string):Promise<ReviewSession>{if(!isTauri())throw new Error('Review requires the desktop runtime.');return invoke<ReviewSession>('resume_review_session',{sessionId})}
export async function getReviewSession(sessionId:string):Promise<ReviewSession|null>{if(!isTauri())throw new Error('Review requires the desktop runtime.');return invoke<ReviewSession|null>('get_review_session',{sessionId})}
export async function abandonReviewSession(sessionId:string):Promise<ReviewSession>{if(!isTauri())throw new Error('Review requires the desktop runtime.');return invoke<ReviewSession>('abandon_review_session',{sessionId})}
export async function submitReviewItem(sessionId:string,itemId:string,outcome:ReviewOutcome):Promise<ReviewSubmitResult>{if(!isTauri())throw new Error('Review requires the desktop runtime.');return invoke<ReviewSubmitResult>('submit_review_item',{request:{sessionId,itemId,outcome}})}
export async function listRecentReviewSessions(limit=5):Promise<ReviewSessionSummary[]>{if(!isTauri())throw new Error('Review requires the desktop runtime.');return invoke<ReviewSessionSummary[]>('list_recent_review_sessions',{limit})}
export async function subscribeReviewChanges(handler:()=>void):Promise<UnlistenFn>{if(!isTauri())return()=>undefined;return listen('english-ai-coach:review-changed',handler)}
