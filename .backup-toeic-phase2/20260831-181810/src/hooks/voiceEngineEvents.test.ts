import { describe, expect, it } from 'vitest'
import type { CorrectionCandidate, TranscriptMessage, VoiceEngineEvent } from '../types'
import { correctionCountForDisplay, reduceVoiceEngineEvent } from '../stores/conversation'

const base = { state: 'LISTENING' as const, messages: [] as TranscriptMessage[], correctionCandidates: [], error: null, streamedTeacherText: '', activeGenerationId: null }

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
    const studentMessage: TranscriptMessage = {
      id: 'student-1', lessonId: 'lesson-1', sequenceIndex: 1, turnIndex: 1,
      role: 'student', text: 'Hi teacher.', source: 'whisper', engineEventType: 'transcript', createdAt: 'now',
    }
    const teacherMessage: TranscriptMessage = {
      id: 'teacher-1', lessonId: 'lesson-1', sequenceIndex: 2, turnIndex: 1,
      role: 'teacher', text: 'Hello! How are you?', source: 'ollama', engineEventType: 'teacher_response', createdAt: 'now',
    }
    const student = reduceVoiceEngineEvent(base, { type: 'transcript', text: 'Hi teacher.', message: studentMessage })
    const teacher = reduceVoiceEngineEvent(
      { ...base, messages: student.messages ?? [] },
      { type: 'teacher_response', text: 'Hello! How are you?', message: teacherMessage },
    )
    expect(teacher.messages?.map(({ role, text }) => ({ role, text }))).toEqual([
      { role: 'student', text: 'Hi teacher.' },
      { role: 'teacher', text: 'Hello! How are you?' },
    ])
  })

  it('does not invent UI messages for a filtered technical transcript', () => {
    expect(reduceVoiceEngineEvent(base, { type: 'transcript', text: '[INAUDIBLE]' })).toEqual({})
  })

  it('associates a persisted correction candidate with its teacher message', () => {
    const message: TranscriptMessage = {
      id: 'teacher-1', lessonId: 'lesson-1', sequenceIndex: 2, turnIndex: 1,
      role: 'teacher', text: 'Small correction: say cooking.', source: 'ollama', engineEventType: 'teacher_response', createdAt: 'now',
    }
    const correctionCandidate: CorrectionCandidate = {
      id: 'correction-1', lessonId: 'lesson-1', studentMessageId: 'student-1', teacherMessageId: message.id,
      studentText: 'I like cook.', teacherResponseText: message.text, detectionMethod: 'teacher_cue_v1', createdAt: 'now',
    }
    const result = reduceVoiceEngineEvent(base, {
      type: 'teacher_response', text: message.text, message, correctionCandidate,
    })
    expect(result.correctionCandidates).toEqual([correctionCandidate])
  })

  it('maps bridge errors without throwing', () => {
    expect(reduceVoiceEngineEvent(base, { type: 'error', message: 'Microphone unavailable' })).toEqual({
      state: 'ERROR',
      error: 'Microphone unavailable',
      streamedTeacherText: '',
      activeGenerationId: null,
    })
  })

  it('uses one transient draft and commits only the persisted final response', () => {
    const started = { ...base, ...reduceVoiceEngineEvent(base, { type: 'teacher_stream_started', generationId: 'turn-a' }) }
    const delta = { ...started, ...reduceVoiceEngineEvent(started, { type: 'teacher_response_delta', generationId: 'turn-a', delta: 'Hello', text: 'Hello there.' }) }
    expect(delta.streamedTeacherText).toBe('Hello there.')
    expect(delta.messages).toHaveLength(0)
    const message: TranscriptMessage = { id: 'teacher-a', lessonId: 'lesson-1', sequenceIndex: 2, turnIndex: 1, role: 'teacher', text: 'Hello there.', source: 'ollama', engineEventType: 'teacher_response', createdAt: 'now' }
    const committed = reduceVoiceEngineEvent(delta, { type: 'teacher_response', generationId: 'turn-a', text: message.text, message })
    expect(committed.messages).toEqual([message])
    expect(committed.streamedTeacherText).toBe('')
  })

  it('drops stale deltas and clears an undelivered cancelled draft', () => {
    const active = { ...base, activeGenerationId: 'turn-b', streamedTeacherText: 'Current' }
    expect(reduceVoiceEngineEvent(active, { type: 'teacher_response_delta', generationId: 'turn-a', delta: 'Old', text: 'Old' })).toEqual({})
    expect(reduceVoiceEngineEvent(active, { type: 'teacher_cancelled', generationId: 'turn-b', deliveredText: '' })).toMatchObject({ state: 'TEACHER_CANCELLING', streamedTeacherText: '', activeGenerationId: null })
  })

  it('recovers from a turn-scoped error without failing the lesson', () => {
    expect(reduceVoiceEngineEvent(base, { type: 'error', message: 'stream dropped', recoverable: true })).toMatchObject({ state: 'LISTENING', error: 'stream dropped' })
  })

  it('uses live candidates while active and the persisted summary after End Lesson', () => {
    const candidates = [{ id: 'one' }, { id: 'two' }] as CorrectionCandidate[]
    expect(correctionCountForDisplay(null, candidates)).toBe(2)
    expect(correctionCountForDisplay({
      lessonId: 'lesson-1', status: 'completed', startedAt: 'start', endedAt: 'end',
      durationSeconds: 271, studentTurns: 9, teacherTurns: 9, correctionCandidates: 4,
    }, [])).toBe(4)
  })
})
