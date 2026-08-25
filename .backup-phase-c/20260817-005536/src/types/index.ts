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
