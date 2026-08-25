import { create } from 'zustand'
import type { ConversationState, CorrectionCandidate, Lesson, LessonAnalysis, LessonAnalysisStatus, LessonSummary, PipelineMetrics, TranscriptMessage, VoiceEngineEvent, VoiceEngineState } from '../types'

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
  lesson: Lesson | null
  correctionCandidates: CorrectionCandidate[]
  summary: LessonSummary | null
  voiceEngineState: VoiceEngineState
  analysis: LessonAnalysis | null
  analysisStatus: LessonAnalysisStatus | null
  analysisError: string | null
  metrics: PipelineMetrics | null
  error: string | null
  transition: (next: ConversationState) => void
  addMessage: (message: Pick<TranscriptMessage, 'role' | 'text'>) => void
  setLesson: (lesson: Lesson) => void
  restoreLesson: (lesson: Lesson, messages: TranscriptMessage[], corrections: CorrectionCandidate[]) => void
  setSummary: (summary: LessonSummary) => void
  setVoiceEngineState: (state: VoiceEngineState) => void
  setAnalysisStatus: (status: LessonAnalysisStatus) => void
  setAnalysis: (analysis: LessonAnalysis) => void
  failAnalysis: (message: string) => void
  setMetrics: (metrics: PipelineMetrics) => void
  fail: (message: string) => void
  reset: () => void
  beginEnding: () => void
  applyVoiceEngineEvent: (event: VoiceEngineEvent) => void
}

export const useConversationStore = create<ConversationStore>((set, get) => ({
  state: 'IDLE', messages: [], lesson: null, correctionCandidates: [], summary: null, voiceEngineState: 'stopped', analysis: null, analysisStatus: null, analysisError: null, metrics: null, error: null,
  transition: (next) => {
    const current = get().state
    if (!ALLOWED[current].includes(next)) throw new Error(`Invalid conversation transition: ${current} → ${next}`)
    set({ state: next, error: null })
  },
  addMessage: (message) => set((store) => ({ messages: [...store.messages, createLegacyMessage(message.role, message.text)] })),
  setLesson: (lesson) => set({ lesson }),
  restoreLesson: (lesson, messages, correctionCandidates) => set({
    lesson, messages, correctionCandidates,
    state: lesson.status === 'failed' ? 'ERROR' : ['completed', 'interrupted'].includes(lesson.status) ? 'COMPLETED' : 'PREPARING',
    error: lesson.errorMessage,
  }),
  setSummary: (summary) => set((store) => ({
    summary,
    lesson: store.lesson ? { ...store.lesson, status: summary.status, endedAt: summary.endedAt, durationSeconds: summary.durationSeconds, studentTurnCount: summary.studentTurns, teacherTurnCount: summary.teacherTurns, correctionCount: summary.correctionCandidates } : null,
    state: summary.status === 'failed' ? 'ERROR' : 'COMPLETED',
  })),
  setVoiceEngineState: (voiceEngineState) => set({ voiceEngineState }),
  setAnalysisStatus: (analysisStatus) => set({ analysisStatus, analysisError: null }),
  setAnalysis: (analysis) => set({ analysis, analysisStatus: analysis.status, analysisError: analysis.errorMessage }),
  failAnalysis: (analysisError) => set({ analysisStatus: 'failed', analysisError }),
  setMetrics: (metrics) => set({ metrics }),
  fail: (message) => set({ state: 'ERROR', error: message }),
  reset: () => set({ state: 'IDLE', messages: [], lesson: null, correctionCandidates: [], summary: null, voiceEngineState: 'stopped', analysis: null, analysisStatus: null, analysisError: null, metrics: null, error: null }),
  beginEnding: () => set({ state: 'ENDING', error: null }),
  applyVoiceEngineEvent: (event) => set((store) => reduceVoiceEngineEvent(store, event)),
}))

export function canTransition(from: ConversationState, to: ConversationState): boolean {
  return ALLOWED[from].includes(to)
}

interface VoiceEventState {
  state: ConversationState
  messages: TranscriptMessage[]
  correctionCandidates: CorrectionCandidate[]
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
      return event.message ? { messages: appendUnique(store.messages, event.message) } : {}
    case 'teacher_thinking':
      return { state: 'TEACHER_THINKING', error: null }
    case 'teacher_response':
      return {
        messages: event.message ? appendUnique(store.messages, event.message) : store.messages,
        correctionCandidates: event.correctionCandidate
          ? appendUnique(store.correctionCandidates, event.correctionCandidate)
          : store.correctionCandidates,
      }
    case 'teacher_speaking':
      return { state: 'TEACHER_SPEAKING', error: null }
    case 'error':
      return { state: 'ERROR', error: event.message }
    case 'engine_stopped':
      return { state: store.state === 'IDLE' || store.state === 'ERROR' ? store.state : 'COMPLETED' }
  }
}

function appendUnique<T extends { id: string }>(items: T[], item: T): T[] {
  return items.some((current) => current.id === item.id) ? items : [...items, item]
}

export function correctionCountForDisplay(
  summary: LessonSummary | null,
  correctionCandidates: CorrectionCandidate[],
): number {
  return summary?.correctionCandidates ?? correctionCandidates.length
}

function createLegacyMessage(role: TranscriptMessage['role'], text: string): TranscriptMessage {
  return {
    role, text, id: crypto.randomUUID(), createdAt: new Date().toISOString(),
    lessonId: 'legacy-pipeline', sequenceIndex: 0, turnIndex: 0,
    source: 'legacy', engineEventType: 'legacy_pipeline',
  }
}
