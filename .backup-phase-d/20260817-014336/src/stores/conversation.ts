import { create } from 'zustand'
import type { ConversationState, PipelineMetrics, TranscriptMessage, VoiceEngineEvent } from '../types'

const ALLOWED: Record<ConversationState, ConversationState[]> = {
  IDLE: ['PREPARING', 'ERROR'], PREPARING: ['LISTENING', 'ERROR'],
  LISTENING: ['STUDENT_SPEAKING', 'PAUSED', 'ENDING', 'ERROR'],
  STUDENT_SPEAKING: ['TRANSCRIBING', 'LISTENING', 'ENDING', 'ERROR'],
  TRANSCRIBING: ['TEACHER_THINKING', 'ENDING', 'ERROR'],
  TEACHER_THINKING: ['TEACHER_SPEAKING', 'LISTENING', 'ENDING', 'ERROR'],
  TEACHER_SPEAKING: ['LISTENING', 'STUDENT_SPEAKING', 'PAUSED', 'ENDING', 'ERROR'],
  PAUSED: ['LISTENING', 'ENDING', 'ERROR'], ENDING: ['ANALYZING', 'COMPLETED', 'ERROR'],
  ANALYZING: ['COMPLETED', 'ERROR'], COMPLETED: ['IDLE', 'PREPARING'], ERROR: ['IDLE', 'PREPARING'],
}

interface ConversationStore {
  state: ConversationState
  messages: TranscriptMessage[]
  metrics: PipelineMetrics | null
  error: string | null
  transition: (next: ConversationState) => void
  addMessage: (message: Omit<TranscriptMessage, 'id' | 'createdAt'>) => void
  setMetrics: (metrics: PipelineMetrics) => void
  fail: (message: string) => void
  reset: () => void
  beginEnding: () => void
  applyVoiceEngineEvent: (event: VoiceEngineEvent) => void
}

export const useConversationStore = create<ConversationStore>((set, get) => ({
  state: 'IDLE', messages: [], metrics: null, error: null,
  transition: (next) => {
    const current = get().state
    if (!ALLOWED[current].includes(next)) throw new Error(`Invalid conversation transition: ${current} → ${next}`)
    set({ state: next, error: null })
  },
  addMessage: (message) => set((store) => ({ messages: [...store.messages, { ...message, id: crypto.randomUUID(), createdAt: new Date().toISOString() }] })),
  setMetrics: (metrics) => set({ metrics }),
  fail: (message) => set({ state: 'ERROR', error: message }),
  reset: () => set({ state: 'IDLE', messages: [], metrics: null, error: null }),
  beginEnding: () => set({ state: 'ENDING', error: null }),
  applyVoiceEngineEvent: (event) => set((store) => reduceVoiceEngineEvent(store, event)),
}))

export function canTransition(from: ConversationState, to: ConversationState): boolean {
  return ALLOWED[from].includes(to)
}

interface VoiceEventState {
  state: ConversationState
  messages: TranscriptMessage[]
  error: string | null
}

export function reduceVoiceEngineEvent(store: VoiceEventState, event: VoiceEngineEvent): Partial<VoiceEventState> {
  switch (event.type) {
    case 'engine_started':
    case 'calibrating':
      return { state: 'PREPARING', error: null }
    case 'calibrated':
    case 'listening':
    case 'teacher_finished':
      return { state: 'LISTENING', error: null }
    case 'student_speaking':
      return { state: 'STUDENT_SPEAKING', error: null }
    case 'speech_finished':
    case 'transcribing':
      return { state: 'TRANSCRIBING', error: null }
    case 'transcript':
      return { messages: [...store.messages, createMessage('student', event.text)] }
    case 'teacher_thinking':
      return { state: 'TEACHER_THINKING', error: null }
    case 'teacher_response':
      return { messages: [...store.messages, createMessage('teacher', event.text)] }
    case 'teacher_speaking':
      return { state: 'TEACHER_SPEAKING', error: null }
    case 'error':
      return { state: 'ERROR', error: event.message }
    case 'engine_stopped':
      return { state: store.state === 'IDLE' || store.state === 'ERROR' ? store.state : 'COMPLETED' }
  }
}

function createMessage(role: TranscriptMessage['role'], text: string): TranscriptMessage {
  return { role, text, id: crypto.randomUUID(), createdAt: new Date().toISOString() }
}
