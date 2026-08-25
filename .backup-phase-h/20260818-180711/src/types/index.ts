export type ConversationState =
  | 'IDLE' | 'PREPARING' | 'LISTENING' | 'STUDENT_SPEAKING'
  | 'TRANSCRIBING' | 'TEACHER_THINKING' | 'TEACHER_SPEAKING'
  | 'PAUSED' | 'ENDING' | 'ANALYZING' | 'COMPLETED' | 'ERROR'

export type ComponentName = 'ollama' | 'llm' | 'whisper' | 'vad' | 'piper' | 'microphone' | 'speaker' | 'sqlite'

export interface ComponentHealth {
  name: ComponentName
  label: string
  ready: boolean
  detail: string
  repairHint?: string
}

export interface Diagnostics {
  components: ComponentHealth[]
  dataDirectory: string
  offlineReady: boolean
  platform: string
}

export interface OptionalLocalModelProbe {
  name: string
  found: boolean
  path: string
}

export interface LocalVoiceEngineProbe {
  projectRoot: string
  localAiRoot: string
  whisper: {
    cliFound: boolean
    cliPath: string
    streamFound: boolean
    streamPath: string
    modelFound: boolean
    modelPath: string
    modelName: string
    threads: number
    additionalModels: OptionalLocalModelProbe[]
  }
  ollama: {
    reachable: boolean
    baseUrl: string
    modelFound: boolean
    modelName: string
  }
  piper: {
    pythonFound: boolean
    pythonPath: string
    installed: boolean
    version: string | null
    voiceFound: boolean
    voiceConfigFound: boolean
    voiceModelPath: string
    voiceConfigPath: string
    voiceName: string
  }
  voiceDefaults: {
    whisperModel: string
    whisperThreads: number
    silenceToStopSeconds: number
    preRollSeconds: number
    startVoiceBlocks: number
    minimumVoiceThreshold: number
    noiseMultiplier: number
    piperVoice: string
    ttsStartSilenceSeconds: number
    ollamaModel: string
    ollamaThinking: boolean
  }
  optionalComponents: {
    sileroFound: boolean
    sileroPath: string
  }
  offlineReady: boolean
  problems: string[]
}

export type VoiceEngineState = 'stopped' | 'starting' | 'running' | 'stopping' | 'error'

export interface VoiceEngineStatus {
  state: VoiceEngineState
  processId: number | null
}

export type LessonStatus = 'starting' | 'active' | 'completed' | 'interrupted' | 'failed'

export interface Lesson {
  id: string
  startedAt: string
  endedAt: string | null
  status: LessonStatus
  topic: string | null
  mode: string
  durationSeconds: number | null
  studentTurnCount: number
  teacherTurnCount: number
  correctionCount: number
  whisperModel: string
  whisperThreads: number
  ollamaModel: string
  piperVoice: string
  voiceEngineVersion: string
  errorMessage: string | null
  createdAt: string
  updatedAt: string
}

export interface TranscriptMessage {
  id: string
  lessonId: string
  sequenceIndex: number
  turnIndex: number
  role: 'student' | 'teacher'
  text: string
  source: string
  engineEventType: string
  createdAt: string
}

export interface CorrectionCandidate {
  id: string
  lessonId: string
  studentMessageId: string
  teacherMessageId: string
  studentText: string
  teacherResponseText: string
  detectionMethod: string
  createdAt: string
}

export interface LessonSummary {
  lessonId: string
  status: LessonStatus
  startedAt: string
  endedAt: string | null
  durationSeconds: number | null
  studentTurns: number
  teacherTurns: number
  correctionCandidates: number
}

export interface StartLessonResult {
  lessonId: string
  lessonStatus: LessonStatus
  voiceEngineState: VoiceEngineState
  lesson: Lesson
}

export type LessonAnalysisStatus = 'pending' | 'running' | 'completed' | 'failed' | 'insufficient_data'

export interface LessonAnalysisScores {
  fluency: number
  grammar: number
  vocabulary: number
  comprehension: number
  interaction: number
  pronunciation: null
}

export interface LessonAnalysisStrength {
  title: string
  evidence: string
}

export interface LessonAnalysisImprovement {
  area: string
  title: string
  explanation: string
  exampleFromLesson: string
  betterAlternative: string
}

export type LessonAnalysisCorrectionCategory =
  | 'grammar' | 'vocabulary' | 'word_choice' | 'verb_tense' | 'preposition'
  | 'article' | 'word_order' | 'naturalness' | 'other'

export interface LessonAnalysisCorrection {
  original: string
  corrected: string
  explanation: string
  category: LessonAnalysisCorrectionCategory
}

export interface LessonAnalysisNaturalAlternative {
  original: string
  alternative: string
}

export interface LessonAnalysisVocabulary {
  wordOrPhrase: string
  meaning: string
  example: string
}

export interface LessonAnalysisRecurringPattern {
  pattern: string
  count: number
  explanation: string
}

export interface LessonAnalysis {
  id: string
  lessonId: string
  status: LessonAnalysisStatus
  schemaVersion: number
  promptVersion: number
  analyzerModel: string
  startedAt: string | null
  completedAt: string | null
  overallScore: number | null
  scores: LessonAnalysisScores | null
  strengths: LessonAnalysisStrength[]
  priorityImprovements: LessonAnalysisImprovement[]
  corrections: LessonAnalysisCorrection[]
  naturalAlternatives: LessonAnalysisNaturalAlternative[]
  vocabulary: LessonAnalysisVocabulary[]
  recurringPatterns: LessonAnalysisRecurringPattern[]
  nextLessonRecommendations: string[]
  summary: string | null
  pronunciationAvailable: boolean
  errorMessage: string | null
  createdAt: string
  updatedAt: string
}

export type LessonHistoryFilter = 'all' | 'completed' | 'interrupted' | 'analyzed' | 'unanalyzed'

export interface LessonHistoryItem {
  id: string
  startedAt: string
  endedAt: string | null
  durationSeconds: number | null
  status: LessonStatus
  topic: string | null
  mode: string
  studentTurnCount: number
  teacherTurnCount: number
  correctionCount: number
  analysisStatus: LessonAnalysisStatus | null
  overallScore: number | null
}

export interface LessonHistoryPage {
  items: LessonHistoryItem[]
  total: number
  limit: number
  offset: number
}

export interface DashboardLatestAnalysis {
  lessonId: string
  startedAt: string
  durationSeconds: number | null
  overallScore: number
  scores: LessonAnalysisScores
}

export interface DashboardSummary {
  totalLessons: number
  completedLessons: number
  totalPracticeSeconds: number | null
  totalStudentTurns: number
  totalCorrections: number
  analyzedLessons: number
  averageOverallScore: number | null
  latestLesson: LessonHistoryItem | null
  latestAnalyzedLesson: DashboardLatestAnalysis | null
  latestRecommendation: string | null
}

export type ScoreDimension = 'fluency' | 'grammar' | 'vocabulary' | 'comprehension' | 'interaction'

export interface ProgressAverages {
  overall: number
  fluency: number
  grammar: number
  vocabulary: number
  comprehension: number
  interaction: number
}

export interface ProgressPoint {
  lessonId: string
  date: string
  durationSeconds: number | null
  overall: number
  fluency: number
  grammar: number
  vocabulary: number
  comprehension: number
  interaction: number
}

export interface ProgressOverview {
  analyzedLessonCount: number
  averages: ProgressAverages | null
  strongestAreas: ScoreDimension[]
  focusAreas: ScoreDimension[]
  points: ProgressPoint[]
  latestRecommendation: string | null
}

export interface LessonDetails {
  lesson: Lesson
  messages: TranscriptMessage[]
  correctionCandidates: CorrectionCandidate[]
  analysis: LessonAnalysis | null
}

export type VocabularyStatus = 'new' | 'learning' | 'known'
export type VocabularyFilter = 'all' | VocabularyStatus
export type VocabularySort = 'recently_seen' | 'first_seen' | 'most_frequent' | 'alphabetical'

export interface VocabularySummary {
  total: number
  new: number
  learning: number
  known: number
  contributingLessons: number
}

export interface LearningMemorySummary {
  vocabularyTotal: number
  vocabularyNew: number
  vocabularyLearning: number
  vocabularyKnown: number
  lessonsContributingVocabulary: number
  recurringMistakesConfirmed: number
}

export interface VocabularyItem {
  id: string
  text: string
  meaning: string
  status: VocabularyStatus
  firstSeenAt: string
  lastSeenAt: string
  lessonCount: number
  occurrenceCount: number
  latestExample: string | null
}

export interface VocabularyPage {
  items: VocabularyItem[]
  total: number
  limit: number
  offset: number
}

export interface VocabularyOccurrence {
  lessonId: string
  lessonDate: string
  example: string
  occurrenceCount: number
}

export interface VocabularyItemDetails {
  item: VocabularyItem
  occurrences: VocabularyOccurrence[]
}

export interface RecurringMistake {
  id: string
  category: LessonAnalysisCorrectionCategory
  title: string
  explanation: string
  lessonCount: number
  occurrenceCount: number
  firstSeenAt: string
  lastSeenAt: string
  status: 'active' | 'improving' | 'resolved'
}

export interface MistakeOccurrence {
  lessonId: string
  lessonDate: string
  original: string
  corrected: string
  explanation: string
}

export interface RecurringMistakeDetails {
  mistake: RecurringMistake
  occurrences: MistakeOccurrence[]
}

export interface LearningMemorySyncResult {
  synchronized: number
  failed: number
  errors: string[]
}

export type VoiceEngineEvent =
  | { type: 'engine_started' }
  | { type: 'calibrating' }
  | { type: 'calibrated'; voiceThreshold: number }
  | { type: 'listening' }
  | { type: 'student_speaking' }
  | { type: 'speech_finished' }
  | { type: 'transcribing' }
  | { type: 'transcript'; text: string; message?: TranscriptMessage }
  | { type: 'teacher_thinking' }
  | { type: 'teacher_response'; text: string; message?: TranscriptMessage; correctionCandidate?: CorrectionCandidate }
  | { type: 'teacher_speaking' }
  | { type: 'teacher_finished' }
  | { type: 'error'; message: string }
  | { type: 'engine_stopped' }

export interface PipelineMetrics {
  sttMs: number
  llmMs: number
  ttsMs: number
  totalMs: number
}

export interface PipelineResult {
  studentText: string
  teacherText: string
  speechAudioBase64: string | null
  audioMimeType: string | null
  metrics: PipelineMetrics
}

export interface OllamaModel {
  name: string
  size: number
  modifiedAt: string
}
