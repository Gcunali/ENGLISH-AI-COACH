import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Diagnostics, LocalVoiceEngineProbe, OllamaModel, VoiceEngineEvent, VoiceEngineStatus } from '../types'

function isTauri(): boolean {
  return '__TAURI_INTERNALS__' in window
}

export async function getDiagnostics(): Promise<Diagnostics> {
  if (!isTauri()) {
    return {
      components: [
        { name: 'ollama', label: 'Ollama', ready: false, detail: 'Desktop runtime required' },
        { name: 'llm', label: 'Language model', ready: false, detail: 'qwen3.5:4b not detected' },
        { name: 'whisper', label: 'Whisper', ready: false, detail: 'Desktop runtime required' },
        { name: 'vad', label: 'Voice activity detection', ready: true, detail: 'RMS VAD configured; Silero is optional' },
        { name: 'piper', label: 'Piper voice', ready: false, detail: 'Desktop runtime required' },
        { name: 'microphone', label: 'Microphone', ready: false, detail: 'Not tested yet' },
        { name: 'speaker', label: 'Speaker', ready: false, detail: 'Not tested yet' },
        { name: 'sqlite', label: 'Local database', ready: false, detail: 'Desktop runtime required' },
      ],
      dataDirectory: 'Desktop runtime only', offlineReady: false, platform: navigator.platform,
    }
  }
  return invoke<Diagnostics>('diagnostics')
}

export async function probeLocalVoiceEngine(): Promise<LocalVoiceEngineProbe> {
  if (!isTauri()) {
    return {
      projectRoot: 'Desktop runtime only',
      localAiRoot: 'Desktop runtime only',
      whisper: {
        cliFound: false,
        cliPath: '',
        streamFound: false,
        streamPath: '',
        modelFound: false,
        modelPath: '',
        modelName: 'ggml-small.en-q5_1.bin',
        threads: 12,
        additionalModels: [],
      },
      ollama: {
        reachable: false,
        baseUrl: 'http://127.0.0.1:11434',
        modelFound: false,
        modelName: 'qwen3.5:4b',
      },
      piper: {
        pythonFound: false,
        pythonPath: '',
        installed: false,
        version: null,
        voiceFound: false,
        voiceConfigFound: false,
        voiceModelPath: '',
        voiceConfigPath: '',
        voiceName: 'en_US-lessac-medium',
      },
      voiceDefaults: {
        whisperModel: 'ggml-small.en-q5_1.bin',
        whisperThreads: 12,
        silenceToStopSeconds: 3.5,
        preRollSeconds: 0.4,
        startVoiceBlocks: 3,
        minimumVoiceThreshold: 350,
        noiseMultiplier: 3,
        piperVoice: 'en_US-lessac-medium',
        ttsStartSilenceSeconds: 0.5,
        ollamaModel: 'qwen3.5:4b',
        ollamaThinking: false,
      },
      optionalComponents: { sileroFound: false, sileroPath: '' },
      offlineReady: false,
      problems: ['Local voice engine probe requires the desktop runtime.'],
    }
  }
  return invoke<LocalVoiceEngineProbe>('probe_local_voice_engine')
}

export async function startVoiceEngine(): Promise<VoiceEngineStatus> {
  if (!isTauri()) throw new Error('The local voice engine requires the desktop runtime.')
  return invoke<VoiceEngineStatus>('start_voice_engine')
}

export async function stopVoiceEngine(): Promise<VoiceEngineStatus> {
  if (!isTauri()) return { state: 'stopped', processId: null }
  return invoke<VoiceEngineStatus>('stop_voice_engine')
}

export async function getVoiceEngineState(): Promise<VoiceEngineStatus> {
  if (!isTauri()) return { state: 'stopped', processId: null }
  return invoke<VoiceEngineStatus>('get_voice_engine_state')
}

export async function subscribeVoiceEngineEvents(
  handler: (event: VoiceEngineEvent) => void,
): Promise<UnlistenFn> {
  if (!isTauri()) return () => undefined
  return listen<VoiceEngineEvent>('voice-engine-event', (event) => handler(event.payload))
}

export async function transcribeAudio(audioBase64: string): Promise<{ text: string; elapsedMs: number }> {
  if (!isTauri()) throw new Error('Local speech recognition is available in the desktop app.')
  return invoke('transcribe_audio', { audioBase64 })
}

export async function chatTeacher(text: string, model: string): Promise<{ text: string; elapsedMs: number }> {
  if (!isTauri()) throw new Error('Local AI is available in the desktop app.')
  return invoke('chat_teacher', { text, model })
}

export async function synthesizeSpeech(text: string): Promise<{ audioBase64: string; mimeType: string; elapsedMs: number }> {
  if (!isTauri()) throw new Error('Local speech synthesis is available in the desktop app.')
  return invoke('synthesize_speech', { text })
}

export async function listOllamaModels(): Promise<OllamaModel[]> {
  if (!isTauri()) return []
  return invoke<OllamaModel[]>('list_ollama_models')
}
