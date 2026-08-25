import { useCallback, useEffect, useRef } from 'react'
import { analyzeLesson, getLessonAnalysis, retryLessonAnalysis } from '../services/native'
import { useConversationStore } from '../stores/conversation'

export function useLessonAnalysis() {
  const lesson = useConversationStore((store) => store.lesson)
  const summary = useConversationStore((store) => store.summary)
  const analysis = useConversationStore((store) => store.analysis)
  const status = useConversationStore((store) => store.analysisStatus)
  const error = useConversationStore((store) => store.analysisError)
  const requestedLessons = useRef(new Set<string>())

  useEffect(() => {
    if (!lesson || summary?.status !== 'completed' || requestedLessons.current.has(lesson.id)) return
    requestedLessons.current.add(lesson.id)
    let disposed = false
    void (async () => {
      try {
        const existing = await getLessonAnalysis(lesson.id)
        if (disposed) return
        if (existing) {
          useConversationStore.getState().setAnalysis(existing)
          return
        }
        useConversationStore.getState().setAnalysisStatus('pending')
        useConversationStore.getState().setAnalysisStatus('running')
        const completed = await analyzeLesson(lesson.id)
        if (!disposed) useConversationStore.getState().setAnalysis(completed)
      } catch (requestError) {
        if (disposed) return
        const persisted = await getLessonAnalysis(lesson.id).catch(() => null)
        if (persisted) {
          useConversationStore.getState().setAnalysis(persisted)
        } else {
          useConversationStore.getState().failAnalysis(
            requestError instanceof Error ? requestError.message : String(requestError),
          )
        }
      }
    })()
    return () => { disposed = true }
  }, [lesson, summary])

  const retry = useCallback(async () => {
    const currentLesson = useConversationStore.getState().lesson
    if (!currentLesson) return
    useConversationStore.getState().setAnalysisStatus('running')
    try {
      const retried = await retryLessonAnalysis(currentLesson.id)
      useConversationStore.getState().setAnalysis(retried)
    } catch (requestError) {
      const persisted = await getLessonAnalysis(currentLesson.id).catch(() => null)
      if (persisted) useConversationStore.getState().setAnalysis(persisted)
      else useConversationStore.getState().failAnalysis(
        requestError instanceof Error ? requestError.message : String(requestError),
      )
    }
  }, [])

  return { analysis, status, error, retry }
}
