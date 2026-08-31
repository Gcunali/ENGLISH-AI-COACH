import { describe, expect, it } from 'vitest'
import { sanitizeForSpeech, takeCompleteSentences } from './speech'

describe('sanitizeForSpeech', () => {
  it('removes visual formatting while preserving spoken content', () => {
    expect(sanitizeForSpeech('## Hello **there**\n- Visit [the guide](https://example.com).')).toBe('Hello there Visit the guide.')
  })
  it('does not speak URLs or code blocks', () => {
    expect(sanitizeForSpeech('See https://example.com. ```const x = 1```')).toBe('See link')
  })
})

describe('takeCompleteSentences', () => {
  it('separates complete sentences from the streaming remainder', () => {
    expect(takeCompleteSentences('How are you? I am still')).toEqual({ complete: ['How are you?'], remainder: 'I am still' })
  })
})
