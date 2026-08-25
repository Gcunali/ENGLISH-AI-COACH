import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { CorrectionCandidate, DashboardSummary, Diagnostics, LearningMemorySummary, LearningMemorySyncResult, Lesson, LessonAnalysis, LessonDetails, LessonHistoryFilter, LessonHistoryPage, LessonSummary, LocalVoiceEngineProbe, OllamaModel, ProgressOverview, RecurringMistake, RecurringMistakeDetails, StartLessonResult, TranscriptMessage, VocabularyFilter, VocabularyItem, VocabularyItemDetails, VocabularyPage, VocabularySort, VocabularyStatus, VocabularySummary, VoiceEngineEvent, VoiceEngineStatus } from '../types'

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

export async function startLesson(): Promise<StartLessonResult> {
  if (!isTauri()) throw new Error('The local voice engine requires the desktop runtime.')
  return invoke<StartLessonResult>('start_lesson')
}

export async function endLesson(): Promise<LessonSummary> {
  if (!isTauri()) throw new Error('Lesson persistence requires the desktop runtime.')
  return invoke<LessonSummary>('end_lesson')
}

export async function getVoiceEngineState(): Promise<VoiceEngineStatus> {
  if (!isTauri()) return { state: 'stopped', processId: null }
  return invoke<VoiceEngineStatus>('get_voice_engine_state')
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
