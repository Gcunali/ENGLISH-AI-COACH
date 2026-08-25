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
