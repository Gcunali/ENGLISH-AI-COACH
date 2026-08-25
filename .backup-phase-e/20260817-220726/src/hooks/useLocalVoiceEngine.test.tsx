// @vitest-environment jsdom

import { act, renderHook } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Lesson, LessonSummary } from '../types'
import { useConversationStore } from '../stores/conversation'

const mocks = vi.hoisted(() => ({
  startLesson: vi.fn(),
  endLesson: vi.fn(),
  getActiveLesson: vi.fn(async () => null),
  getLesson: vi.fn(async () => null),
  getLessonCorrections: vi.fn(async () => []),
  getLessonTranscript: vi.fn(async () => []),
  getVoiceEngineState: vi.fn(async () => ({ state: 'stopped', processId: null })),
  subscribeVoiceEngineEvents: vi.fn(async () => () => undefined),
}))

vi.mock('../services/native', () => mocks)

import { useLocalVoiceEngine } from './useLocalVoiceEngine'

const lesson: Lesson = {
  id: 'lesson-1', startedAt: '2026-08-17T01:00:00.000Z', endedAt: null,
  status: 'starting', topic: null, mode: 'free_conversation', durationSeconds: null,
  studentTurnCount: 0, teacherTurnCount: 0, correctionCount: 0,
  whisperModel: 'whisper.bin', whisperThreads: 12, ollamaModel: 'qwen',
  piperVoice: 'lessac', voiceEngineVersion: 'voice_v2_bridge_v1', errorMessage: null,
  createdAt: '2026-08-17T01:00:00.000Z', updatedAt: '2026-08-17T01:00:00.000Z',
}

const summary: LessonSummary = {
  lessonId: lesson.id, status: 'completed', startedAt: lesson.startedAt,
  endedAt: '2026-08-17T01:02:00.000Z', durationSeconds: 120,
  studentTurns: 2, teacherTurns: 2, correctionCandidates: 1,
}

describe('persisted lesson hook', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useConversationStore.getState().reset()
    mocks.startLesson.mockResolvedValue({
      lesson,
      lessonId: lesson.id,
      lessonStatus: lesson.status,
      voiceEngineState: 'starting',
    })
    mocks.endLesson.mockResolvedValue(summary)
  })

  it('stores the lesson id returned by startLesson', async () => {
    const { result, unmount } = renderHook(() => useLocalVoiceEngine())
    await act(async () => result.current.startLesson())
    expect(useConversationStore.getState().lesson?.id).toBe('lesson-1')
    expect(useConversationStore.getState().voiceEngineState).toBe('starting')
    unmount()
  })

  it('stores the factual summary returned by End Lesson', async () => {
    const { result, unmount } = renderHook(() => useLocalVoiceEngine())
    await act(async () => result.current.startLesson())
    await act(async () => result.current.endLesson())
    expect(useConversationStore.getState().summary).toEqual(summary)
    expect(useConversationStore.getState().lesson?.status).toBe('completed')
    unmount()
  })

  it('restores an interrupted lesson as completed historical state', async () => {
    useConversationStore.getState().restoreLesson(
      { ...lesson, status: 'interrupted', endedAt: '2026-08-17T01:01:00.000Z' },
      [],
      [],
    )
    expect(useConversationStore.getState().state).toBe('COMPLETED')
    expect(useConversationStore.getState().lesson?.status).toBe('interrupted')
  })
})
