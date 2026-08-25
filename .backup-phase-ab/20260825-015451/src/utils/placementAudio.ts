const PLACEMENT_VOICE_THRESHOLD = 0.012
const PLACEMENT_VAD_PADDING_CHUNKS = 8

export function trimPlacementSpeech(chunks: Float32Array[]): Float32Array[] | null {
  const voiced = chunks
    .map((chunk, index) => ({ index, rms: Math.sqrt(chunk.reduce((sum, value) => sum + value * value, 0) / Math.max(1, chunk.length)) }))
    .filter(({ rms }) => rms >= PLACEMENT_VOICE_THRESHOLD)
  if (voiced.length === 0) return null
  const start = Math.max(0, voiced[0].index - PLACEMENT_VAD_PADDING_CHUNKS)
  const end = Math.min(chunks.length, voiced[voiced.length - 1].index + PLACEMENT_VAD_PADDING_CHUNKS + 1)
  return chunks.slice(start, end)
}
