import { describe, expect, it } from 'vitest'
import { trimPlacementSpeech } from './placementAudio'

describe('placement RMS VAD', () => {
  it('rejects silence instead of sending it to Whisper', () => {
    expect(trimPlacementSpeech([new Float32Array(128), new Float32Array(128)])).toBeNull()
  })
  it('keeps voiced samples with bounded surrounding context', () => {
    const chunks = Array.from({ length: 30 }, () => new Float32Array(128))
    chunks[15].fill(0.1)
    const trimmed = trimPlacementSpeech(chunks)
    expect(trimmed).not.toBeNull()
    expect(trimmed!.length).toBe(17)
    expect(trimmed!.some((chunk) => chunk[0] > 0.09)).toBe(true)
  })
})
