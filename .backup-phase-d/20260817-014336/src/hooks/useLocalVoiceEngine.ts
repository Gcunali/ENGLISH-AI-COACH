import { useCallback, useEffect, useRef, useState } from 'react'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getVoiceEngineState, startVoiceEngine, stopVoiceEngine, subscribeVoiceEngineEvents } from '../services/native'
import { useConversationStore } from '../stores/conversation'
import type { VoiceEngineEvent, VoiceEngineState } from '../types'

function stateFromEvent(event: VoiceEngineEvent): VoiceEngineState {
  switch (event.type) {
    case 'engine_started':
      return 'running'
    case 'engine_stopped':
      return 'stopped'
    case 'error':
      return 'error'
    default:
      return 'running'
  }
}

export function useLocalVoiceEngine() {
  const [engineState, setEngineState] = useState<VoiceEngineState>('stopped')
  const subscriptionRef = useRef<Promise<UnlistenFn> | null>(null)

  const ensureSubscription = useCallback(() => {
    if (!subscriptionRef.current) {
      subscriptionRef.current = subscribeVoiceEngineEvents((event) => {
        useConversationStore.getState().applyVoiceEngineEvent(event)
        setEngineState(stateFromEvent(event))
      })
    }
    return subscriptionRef.current
  }, [])

  useEffect(() => {
    let disposed = false
    const subscription = ensureSubscription()
    void getVoiceEngineState().then((status) => {
      if (!disposed) setEngineState(status.state)
    })
    return () => {
      disposed = true
      void subscription.then((unlisten) => unlisten())
      subscriptionRef.current = null
    }
  }, [ensureSubscription])

  const startLesson = useCallback(async () => {
    await ensureSubscription()
    const store = useConversationStore.getState()
    store.reset()
    store.transition('PREPARING')
    setEngineState('starting')
    try {
      const status = await startVoiceEngine()
      setEngineState((current) => current === 'running' ? current : status.state)
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      useConversationStore.getState().fail(message)
      setEngineState('error')
    }
  }, [ensureSubscription])

  const endLesson = useCallback(async () => {
    useConversationStore.getState().beginEnding()
    setEngineState('stopping')
    try {
      const status = await stopVoiceEngine()
      setEngineState(status.state)
      if (useConversationStore.getState().state === 'ENDING') {
        useConversationStore.getState().applyVoiceEngineEvent({ type: 'engine_stopped' })
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      useConversationStore.getState().fail(message)
      setEngineState('error')
    }
  }, [])

  return { engineState, startLesson, endLesson }
}
