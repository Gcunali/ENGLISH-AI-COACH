import { describe, expect, it, vi } from 'vitest'
import type { TranscriptMessage, VoiceEngineEvent } from '../types'
import { reduceVoiceEngineEvent } from '../stores/conversation'

const base = { state: 'LISTENING' as const, messages: [] as TranscriptMessage[], error: null }

describe('Voice Engine event reducer', () => {
  it.each<[VoiceEngineEvent, string]>([
    [{ type: 'calibrating' }, 'PREPARING'],
    [{ type: 'listening' }, 'LISTENING'],
    [{ type: 'speech_finished' }, 'TRANSCRIBING'],
    [{ type: 'teacher_thinking' }, 'TEACHER_THINKING'],
    [{ type: 'teacher_speaking' }, 'TEACHER_SPEAKING'],
  ])('maps $type to %s', (event, expected) => {
    expect(reduceVoiceEngineEvent(base, event).state).toBe(expected)
  })

  it('adds transcript and teacher response messages', () => {
    vi.stubGlobal('crypto', { randomUUID: () => 'event-id' })
    const student = reduceVoiceEngineEvent(base, { type: 'transcript', text: 'Hi teacher.' })
    const teacher = reduceVoiceEngineEvent(
      { ...base, messages: student.messages ?? [] },
      { type: 'teacher_response', text: 'Hello! How are you?' },
    )
    expect(teacher.messages?.map(({ role, text }) => ({ role, text }))).toEqual([
      { role: 'student', text: 'Hi teacher.' },
      { role: 'teacher', text: 'Hello! How are you?' },
    ])
    vi.unstubAllGlobals()
  })

  it('maps bridge errors without throwing', () => {
    expect(reduceVoiceEngineEvent(base, { type: 'error', message: 'Microphone unavailable' })).toEqual({
      state: 'ERROR',
      error: 'Microphone unavailable',
    })
  })
})
