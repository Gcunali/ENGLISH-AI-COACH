export function encodeWavBase64(samples: Float32Array, sampleRate: number): string {
  const bytes = new ArrayBuffer(44 + samples.length * 2)
  const view = new DataView(bytes)
  write(view, 0, 'RIFF')
  view.setUint32(4, 36 + samples.length * 2, true)
  write(view, 8, 'WAVE')
  write(view, 12, 'fmt ')
  view.setUint32(16, 16, true)
  view.setUint16(20, 1, true)
  view.setUint16(22, 1, true)
  view.setUint32(24, sampleRate, true)
  view.setUint32(28, sampleRate * 2, true)
  view.setUint16(32, 2, true)
  view.setUint16(34, 16, true)
  write(view, 36, 'data')
  view.setUint32(40, samples.length * 2, true)
  for (let index = 0; index < samples.length; index += 1) {
    const sample = Math.max(-1, Math.min(1, samples[index]))
    view.setInt16(44 + index * 2, sample < 0 ? sample * 0x8000 : sample * 0x7fff, true)
  }
  const bytesView = new Uint8Array(bytes)
  let binary = ''
  const chunkSize = 0x8000
  for (let index = 0; index < bytesView.length; index += chunkSize) {
    binary += String.fromCharCode(...bytesView.subarray(index, index + chunkSize))
  }
  return btoa(binary)
}

function write(view: DataView, offset: number, value: string): void {
  for (let index = 0; index < value.length; index += 1) view.setUint8(offset + index, value.charCodeAt(index))
}

export function downsample(input: Float32Array, inputRate: number, outputRate = 16_000): Float32Array {
  if (inputRate === outputRate) return input
  const ratio = inputRate / outputRate
  const output = new Float32Array(Math.round(input.length / ratio))
  for (let outIndex = 0; outIndex < output.length; outIndex += 1) {
    const start = Math.floor(outIndex * ratio)
    const end = Math.min(input.length, Math.floor((outIndex + 1) * ratio))
    let sum = 0
    for (let index = start; index < end; index += 1) sum += input[index]
    output[outIndex] = sum / Math.max(1, end - start)
  }
  return output
}
