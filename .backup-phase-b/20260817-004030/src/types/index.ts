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

export interface TranscriptMessage {
  id: string
  role: 'student' | 'teacher'
  text: string
  createdAt: string
}

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
