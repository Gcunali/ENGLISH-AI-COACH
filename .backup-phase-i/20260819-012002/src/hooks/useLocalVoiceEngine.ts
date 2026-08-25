import { useCallback, useEffect, useRef } from 'react'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { endLesson as endPersistedLesson, getActiveLesson, getLatestCompletedLesson, getLesson, getLessonCorrections, getLessonTranscript, getVoiceEngineState, startLesson as startPersistedLesson, subscribeVoiceEngineEvents } from '../services/native'
import { useConversationStore } from '../stores/conversation'
import type { Lesson, LessonSummary, VoiceEngineEvent, VoiceEngineState } from '../types'

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
  const engineState = useConversationStore((store) => store.voiceEngineState)
  const subscriptionRef = useRef<Promise<UnlistenFn> | null>(null)

  const ensureSubscription = useCallback(() => {
    if (!subscriptionRef.current) {
      subscriptionRef.current = subscribeVoiceEngineEvents((event) => {
        const store = useConversationStore.getState()
        store.applyVoiceEngineEvent(event)
        store.setVoiceEngineState(stateFromEvent(event))
        if ((event.type === 'error' || event.type === 'engine_stopped') && store.lesson) {
          void getLesson(store.lesson.id).then((lesson) => {
            if (lesson && ['failed', 'interrupted'].includes(lesson.status)) {
              useConversationStore.getState().setSummary(summaryFromLesson(lesson))
            }
          })
        }
      })
    }
    return subscriptionRef.current
  }, [])

  useEffect(() => {
    let disposed = false
    const subscription = ensureSubscription()
    void getVoiceEngineState().then((status) => {
      if (!disposed) useConversationStore.getState().setVoiceEngineState(status.state)
    })
    void getActiveLesson().then(async (activeLesson) => {
      const lesson = activeLesson ?? await getLatestCompletedLesson()
      if (!disposed && lesson) {
        const [messages, corrections] = await Promise.all([
          getLessonTranscript(lesson.id),
          getLessonCorrections(lesson.id),
        ])
        if (!disposed) {
          const store = useConversationStore.getState()
          store.restoreLesson(lesson, messages, corrections)
          if (lesson.status === 'completed') store.setSummary(summaryFromLesson(lesson), false)
        }
      }
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
    store.setVoiceEngineState('starting')
    try {
      const result = await startPersistedLesson()
      useConversationStore.getState().setLesson(result.lesson)
      const current = useConversationStore.getState().voiceEngineState
      useConversationStore.getState().setVoiceEngineState(current === 'running' ? current : result.voiceEngineState)
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      useConversationStore.getState().fail(message)
      useConversationStore.getState().setVoiceEngineState('error')
    }
  }, [ensureSubscription])

  const endLesson = useCallback(async () => {
    useConversationStore.getState().beginEnding()
    useConversationStore.getState().setVoiceEngineState('stopping')
    try {
      const summary = await endPersistedLesson()
      useConversationStore.getState().setSummary(summary)
      useConversationStore.getState().setVoiceEngineState('stopped')
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      useConversationStore.getState().fail(message)
      useConversationStore.getState().setVoiceEngineState('error')
    }
  }, [])

  return { engineState, startLesson, endLesson }
}

function summaryFromLesson(lesson: Lesson): LessonSummary {
  return {
    lessonId: lesson.id,
    status: lesson.status,
    startedAt: lesson.startedAt,
    endedAt: lesson.endedAt,
    durationSeconds: lesson.durationSeconds,
    studentTurns: lesson.studentTurnCount,
    teacherTurns: lesson.teacherTurnCount,
    correctionCandidates: lesson.correctionCount,
  }
}
