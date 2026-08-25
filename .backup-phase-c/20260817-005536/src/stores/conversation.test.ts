import { describe, expect, it } from 'vitest'
import { canTransition } from './conversation'

describe('conversation state machine', () => {
  it('allows the normal voice pipeline', () => {
    expect(canTransition('LISTENING', 'STUDENT_SPEAKING')).toBe(true)
    expect(canTransition('STUDENT_SPEAKING', 'TRANSCRIBING')).toBe(true)
    expect(canTransition('TRANSCRIBING', 'TEACHER_THINKING')).toBe(true)
  })
  it('rejects impossible jumps', () => expect(canTransition('IDLE', 'TEACHER_SPEAKING')).toBe(false))
})
