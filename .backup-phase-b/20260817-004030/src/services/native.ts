import { invoke } from '@tauri-apps/api/core'
import type { Diagnostics, OllamaModel } from '../types'

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
        { name: 'vad', label: 'Silero VAD', ready: false, detail: 'Model not detected' },
        { name: 'piper', label: 'Piper voice', ready: false, detail: 'Desktop runtime required' },
        { name: 'microphone', label: 'Microphone', ready: false, detail: 'Not tested yet' },
        { name: 'speaker', label: 'Speaker', ready: true, detail: 'Browser audio available' },
        { name: 'sqlite', label: 'Local database', ready: false, detail: 'Desktop runtime required' },
      ],
      dataDirectory: 'Desktop runtime only', offlineReady: false, platform: navigator.platform,
    }
  }
  return invoke<Diagnostics>('diagnostics')
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
