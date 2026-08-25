// @vitest-environment jsdom

import { act, renderHook, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Lesson, LessonAnalysis, LessonSummary } from '../types'
import { useConversationStore } from '../stores/conversation'

const mocks = vi.hoisted(() => ({
  analyzeLesson: vi.fn(),
  getLessonAnalysis: vi.fn(),
  retryLessonAnalysis: vi.fn(),
}))

vi.mock('../services/native', () => mocks)

import { useLessonAnalysis } from './useLessonAnalysis'

const lesson: Lesson = {
  id: 'lesson-1', startedAt: 'start', endedAt: 'end', status: 'completed', topic: null,
  mode: 'free_conversation', durationSeconds: 120, studentTurnCount: 3, teacherTurnCount: 3,
  correctionCount: 1, whisperModel: 'whisper', whisperThreads: 12, ollamaModel: 'qwen3.5:4b',
  piperVoice: 'lessac', voiceEngineVersion: 'voice-v2', errorMessage: null,
  createdAt: 'start', updatedAt: 'end',
}

const summary: LessonSummary = {
  lessonId: lesson.id, status: 'completed', startedAt: 'start', endedAt: 'end',
  durationSeconds: 120, studentTurns: 3, teacherTurns: 3, correctionCandidates: 1,
}

const completed: LessonAnalysis = {
  id: 'analysis-1', lessonId: lesson.id, status: 'completed', schemaVersion: 1, promptVersion: 1,
  analyzerModel: 'qwen3.5:4b', startedAt: 'start', completedAt: 'end', overallScore: 70,
  scores: { fluency: 70, grammar: 60, vocabulary: 65, comprehension: 80, interaction: 75, pronunciation: null },
  strengths: [], priorityImprovements: [], corrections: [], naturalAlternatives: [], vocabulary: [],
  recurringPatterns: [], nextLessonRecommendations: [], summary: 'Resumo', pronunciationAvailable: false,
  errorMessage: null, createdAt: 'start', updatedAt: 'end',
}

describe('useLessonAnalysis', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useConversationStore.getState().reset()
    useConversationStore.getState().restoreLesson(lesson, [], [])
    useConversationStore.getState().setSummary(summary)
  })

  it('starts analysis automatically only after the factual summary exists', async () => {
    mocks.getLessonAnalysis.mockResolvedValue(null)
    mocks.analyzeLesson.mockResolvedValue(completed)
    renderHook(() => useLessonAnalysis())
    await waitFor(() => expect(useConversationStore.getState().analysisStatus).toBe('completed'))
    expect(mocks.analyzeLesson).toHaveBeenCalledWith(lesson.id)
  })

  it('loads an existing completed analysis without calling Ollama again', async () => {
    mocks.getLessonAnalysis.mockResolvedValue(completed)
    renderHook(() => useLessonAnalysis())
    await waitFor(() => expect(useConversationStore.getState().analysis?.id).toBe('analysis-1'))
    expect(mocks.analyzeLesson).not.toHaveBeenCalled()
  })

  it('retries a failed analysis explicitly', async () => {
    const failed = { ...completed, status: 'failed' as const, scores: null, overallScore: null, errorMessage: 'invalid JSON' }
    mocks.getLessonAnalysis.mockResolvedValue(failed)
    mocks.retryLessonAnalysis.mockResolvedValue(completed)
    const { result } = renderHook(() => useLessonAnalysis())
    await waitFor(() => expect(useConversationStore.getState().analysisStatus).toBe('failed'))
    await act(async () => result.current.retry())
    expect(mocks.retryLessonAnalysis).toHaveBeenCalledWith(lesson.id)
    expect(useConversationStore.getState().analysisStatus).toBe('completed')
  })
})
