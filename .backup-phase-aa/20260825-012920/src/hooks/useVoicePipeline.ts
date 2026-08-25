import { useCallback, useEffect, useRef, useState } from 'react'
import { chatTeacher, synthesizeSpeech, transcribeAudio } from '../services/native'
import { useConversationStore } from '../stores/conversation'
import { downsample, encodeWavBase64 } from '../utils/audio'

const MINIMUM_SPEECH_MS = 250
const SILENCE_END_MS = 800
const MAXIMUM_UTTERANCE_MS = 30_000
const SPEECH_THRESHOLD = 0.018
const PRE_ROLL_MS = 320

interface VoicePipeline {
  level: number
  startLesson: () => Promise<void>
  endLesson: () => void
  beginPushToTalk: () => void
  endPushToTalk: () => void
  isReady: boolean
  mode: 'AUTO' | 'PUSH_TO_TALK'
  setMode: (mode: 'AUTO' | 'PUSH_TO_TALK') => void
}

export function useVoicePipeline(model: string): VoicePipeline {
  const [level, setLevel] = useState(0)
  const [mode, setMode] = useState<'AUTO' | 'PUSH_TO_TALK'>('AUTO')
  const [isReady, setReady] = useState(false)
  const contextRef = useRef<AudioContext | null>(null)
  const streamRef = useRef<MediaStream | null>(null)
  const sourceRef = useRef<MediaStreamAudioSourceNode | null>(null)
  const workletRef = useRef<AudioWorkletNode | null>(null)
  const preRollRef = useRef<Float32Array[]>([])
  const samplesRef = useRef<Float32Array[]>([])
  const speechStartRef = useRef(0)
  const lastSpeechRef = useRef(0)
  const voiceCandidateRef = useRef(0)
  const recordingRef = useRef(false)
  const modeRef = useRef(mode)
  const audioRef = useRef<HTMLAudioElement | null>(null)

  useEffect(() => { modeRef.current = mode }, [mode])

  const stopTeacher = useCallback(() => {
    if (audioRef.current) {
      audioRef.current.pause()
      audioRef.current.src = ''
      audioRef.current = null
    }
  }, [])

  const finishUtterance = useCallback(async () => {
    if (!recordingRef.current || samplesRef.current.length === 0) return
    recordingRef.current = false
    const chunks = samplesRef.current
    samplesRef.current = []
    const length = chunks.reduce((sum, chunk) => sum + chunk.length, 0)
    const combined = new Float32Array(length)
    let offset = 0
    for (const chunk of chunks) { combined.set(chunk, offset); offset += chunk.length }
    const context = contextRef.current
    if (!context) return

    const store = useConversationStore.getState()
    try {
      store.transition('TRANSCRIBING')
      const startedAt = performance.now()
      const wav = encodeWavBase64(downsample(combined, context.sampleRate), 16_000)
      const transcription = await transcribeAudio(wav)
      if (!transcription.text.trim()) {
        useConversationStore.getState().transition('LISTENING')
        return
      }
      useConversationStore.getState().addMessage({ role: 'student', text: transcription.text })
      useConversationStore.getState().transition('TEACHER_THINKING')
      const answer = await chatTeacher(transcription.text, model)
      useConversationStore.getState().addMessage({ role: 'teacher', text: answer.text })
      const speech = await synthesizeSpeech(answer.text)
      useConversationStore.getState().setMetrics({
        sttMs: transcription.elapsedMs, llmMs: answer.elapsedMs, ttsMs: speech.elapsedMs,
        totalMs: Math.round(performance.now() - startedAt),
      })
      useConversationStore.getState().transition('TEACHER_SPEAKING')
      const audio = new Audio(`data:${speech.mimeType};base64,${speech.audioBase64}`)
      audioRef.current = audio
      audio.onended = () => {
        audioRef.current = null
        if (useConversationStore.getState().state === 'TEACHER_SPEAKING') useConversationStore.getState().transition('LISTENING')
      }
      await audio.play()
    } catch (error) {
      useConversationStore.getState().fail(error instanceof Error ? error.message : String(error))
    }
  }, [model])

  const beginSpeech = useCallback(() => {
    const state = useConversationStore.getState().state
    if (recordingRef.current || !['LISTENING', 'TEACHER_SPEAKING'].includes(state)) return
    if (state === 'TEACHER_SPEAKING') stopTeacher()
    useConversationStore.getState().transition('STUDENT_SPEAKING')
    recordingRef.current = true
    speechStartRef.current = performance.now()
    lastSpeechRef.current = performance.now()
    samplesRef.current = [...preRollRef.current]
  }, [stopTeacher])

  const onSamples = useCallback((samples: Float32Array) => {
    const rms = Math.sqrt(samples.reduce((sum, value) => sum + value * value, 0) / samples.length)
    setLevel(Math.min(1, rms * 12))
    const now = performance.now()
    const context = contextRef.current
    if (!context) return

    const maxPreRollChunks = Math.ceil((PRE_ROLL_MS / 1000) * context.sampleRate / samples.length)
    preRollRef.current.push(samples)
    if (preRollRef.current.length > maxPreRollChunks) preRollRef.current.shift()

    if (recordingRef.current) samplesRef.current.push(samples)
    if (modeRef.current === 'PUSH_TO_TALK') return

    if (rms >= SPEECH_THRESHOLD) {
      lastSpeechRef.current = now
      if (!voiceCandidateRef.current) voiceCandidateRef.current = now
      if (!recordingRef.current && now - voiceCandidateRef.current >= MINIMUM_SPEECH_MS) beginSpeech()
    } else {
      voiceCandidateRef.current = 0
      if (recordingRef.current && (now - lastSpeechRef.current >= SILENCE_END_MS || now - speechStartRef.current >= MAXIMUM_UTTERANCE_MS)) void finishUtterance()
    }
  }, [beginSpeech, finishUtterance])

  const startLesson = useCallback(async () => {
    const store = useConversationStore.getState()
    if (store.state === 'ERROR' || store.state === 'COMPLETED') store.reset()
    useConversationStore.getState().transition('PREPARING')
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: { channelCount: 1, echoCancellation: true, noiseSuppression: true, autoGainControl: true } })
      const context = new AudioContext({ latencyHint: 'interactive' })
      await context.audioWorklet.addModule('/pcm-capture-processor.js')
      const source = context.createMediaStreamSource(stream)
      const worklet = new AudioWorkletNode(context, 'pcm-capture-processor')
      const silent = context.createGain()
      silent.gain.value = 0
      source.connect(worklet).connect(silent).connect(context.destination)
      worklet.port.onmessage = (event: MessageEvent<Float32Array>) => onSamples(event.data)
      contextRef.current = context
      streamRef.current = stream
      sourceRef.current = source
      workletRef.current = worklet
      setReady(true)
      useConversationStore.getState().transition('LISTENING')
    } catch (error) {
      useConversationStore.getState().fail(`Microphone unavailable: ${error instanceof Error ? error.message : String(error)}`)
    }
  }, [onSamples])

  const endLesson = useCallback(() => {
    stopTeacher()
    if (['LISTENING', 'STUDENT_SPEAKING', 'TEACHER_SPEAKING', 'PAUSED'].includes(useConversationStore.getState().state)) {
      useConversationStore.getState().transition('ENDING')
      useConversationStore.getState().transition('COMPLETED')
    }
    workletRef.current?.disconnect()
    sourceRef.current?.disconnect()
    streamRef.current?.getTracks().forEach((track) => track.stop())
    void contextRef.current?.close()
    contextRef.current = null
    setReady(false)
    setLevel(0)
  }, [stopTeacher])

  useEffect(() => endLesson, [endLesson])

  const beginPushToTalk = useCallback(() => { if (modeRef.current === 'PUSH_TO_TALK') beginSpeech() }, [beginSpeech])
  const endPushToTalk = useCallback(() => { if (modeRef.current === 'PUSH_TO_TALK') void finishUtterance() }, [finishUtterance])

  return { level, startLesson, endLesson, beginPushToTalk, endPushToTalk, isReady, mode, setMode }
}
