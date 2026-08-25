import { create } from 'zustand'
import type { ConversationState, PipelineMetrics, TranscriptMessage } from '../types'

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
}))

export function canTransition(from: ConversationState, to: ConversationState): boolean {
  return ALLOWED[from].includes(to)
}
