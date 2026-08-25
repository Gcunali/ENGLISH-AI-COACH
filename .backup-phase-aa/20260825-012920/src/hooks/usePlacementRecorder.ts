import { useCallback, useEffect, useRef, useState } from 'react'
import { capturePlacementSpeakingResponse } from '../services/native'
import { downsample, encodeWavBase64 } from '../utils/audio'
import { trimPlacementSpeech } from '../utils/placementAudio'

export type PlacementRecordingState = 'idle' | 'listening' | 'transcribing' | 'error'

export function usePlacementRecorder() {
  const [state, setState] = useState<PlacementRecordingState>('idle')
  const [transcript, setTranscript] = useState('')
  const [error, setError] = useState<string | null>(null)
  const context = useRef<AudioContext | null>(null)
  const stream = useRef<MediaStream | null>(null)
  const source = useRef<MediaStreamAudioSourceNode | null>(null)
  const worklet = useRef<AudioWorkletNode | null>(null)
  const samples = useRef<Float32Array[]>([])
  const timer = useRef<number | null>(null)

  const cleanup = useCallback(() => {
    if (timer.current !== null) window.clearTimeout(timer.current)
    timer.current = null
    worklet.current?.disconnect(); source.current?.disconnect()
    stream.current?.getTracks().forEach((track) => track.stop())
    void context.current?.close()
    worklet.current = null; source.current = null; stream.current = null; context.current = null
  }, [])

  const stop = useCallback(async () => {
    const audioContext = context.current
    const chunks = trimPlacementSpeech(samples.current)
    cleanup()
    if (!audioContext || !chunks || chunks.length === 0) { setState('error'); setError('No valid speech was detected. Please record again.'); return }
    setState('transcribing'); setError(null)
    const length = chunks.reduce((sum, chunk) => sum + chunk.length, 0)
    const combined = new Float32Array(length)
    let offset = 0
    for (const chunk of chunks) { combined.set(chunk, offset); offset += chunk.length }
    try {
      const wav = encodeWavBase64(downsample(combined, audioContext.sampleRate), 16_000)
      const result = await capturePlacementSpeakingResponse(wav)
      setTranscript(result.text); setState('idle')
    } catch (value) { setError(value instanceof Error ? value.message : String(value)); setState('error') }
  }, [cleanup])

  const start = useCallback(async () => {
    cleanup(); samples.current = []; setTranscript(''); setError(null)
    try {
      const media = await navigator.mediaDevices.getUserMedia({ audio: { channelCount: 1, echoCancellation: true, noiseSuppression: true, autoGainControl: true } })
      const audioContext = new AudioContext({ latencyHint: 'interactive' })
      await audioContext.audioWorklet.addModule('/pcm-capture-processor.js')
      const input = audioContext.createMediaStreamSource(media)
      const processor = new AudioWorkletNode(audioContext, 'pcm-capture-processor')
      const silent = audioContext.createGain(); silent.gain.value = 0
      input.connect(processor).connect(silent).connect(audioContext.destination)
      processor.port.onmessage = (event: MessageEvent<Float32Array>) => samples.current.push(event.data)
      context.current = audioContext; stream.current = media; source.current = input; worklet.current = processor
      setState('listening')
      timer.current = window.setTimeout(() => { void stop() }, 90_000)
    } catch (value) { cleanup(); setError(`Microphone unavailable: ${value instanceof Error ? value.message : String(value)}`); setState('error') }
  }, [cleanup, stop])

  const retry = useCallback(() => { setTranscript(''); setError(null); setState('idle') }, [])
  useEffect(() => cleanup, [cleanup])
  return { state, transcript, error, start, stop, retry }
}
