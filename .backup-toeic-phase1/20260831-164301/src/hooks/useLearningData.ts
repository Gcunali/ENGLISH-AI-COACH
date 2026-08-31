import { useCallback, useEffect, useState } from 'react'
import { getDashboardSummary, getLearningMemorySummary, getLessonDetails, getProgressOverview, getStudentLearningSummary, getVocabularyItem, getVocabularySummary, listLessons, listRecurringMistakes, listVocabularyItems } from '../services/native'
import type { DashboardSummary, LearningMemorySummary, LessonDetails, LessonHistoryFilter, LessonHistoryPage, ProgressOverview, RecurringMistake, StudentLearningSummary, VocabularyFilter, VocabularyItemDetails, VocabularyPage, VocabularySort, VocabularySummary } from '../types'
import { LEARNING_DATA_CHANGED_EVENT } from '../utils/learningData'

interface QueryState<T> {
  data: T | null
  loading: boolean
  error: string | null
  reload: () => void
}

function useNativeQuery<T>(key: string, loader: () => Promise<T>): QueryState<T> {
  const [data, setData] = useState<T | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [revision, setRevision] = useState(0)
  const reload = useCallback(() => setRevision((value) => value + 1), [])

  useEffect(() => {
    let disposed = false
    setLoading(true)
    setError(null)
    void loader()
      .then((value) => { if (!disposed) setData(value) })
      .catch((reason: unknown) => {
        if (!disposed) setError(reason instanceof Error ? reason.message : String(reason))
      })
      .finally(() => { if (!disposed) setLoading(false) })
    return () => { disposed = true }
    // The stable key intentionally owns loader inputs.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, revision])

  useEffect(() => {
    window.addEventListener(LEARNING_DATA_CHANGED_EVENT, reload)
    return () => window.removeEventListener(LEARNING_DATA_CHANGED_EVENT, reload)
  }, [reload])

  return { data, loading, error, reload }
}

export function useDashboardData(): QueryState<DashboardSummary> {
  return useNativeQuery('dashboard', getDashboardSummary)
}

export function useHistoryData(filter: LessonHistoryFilter, limit: number, offset: number): QueryState<LessonHistoryPage> {
  return useNativeQuery(`history:${filter}:${limit}:${offset}`, () => listLessons(filter, limit, offset))
}

export function useLessonDetailsData(lessonId: string): QueryState<LessonDetails | null> {
  return useNativeQuery(`details:${lessonId}`, () => getLessonDetails(lessonId))
}

export function useProgressData(): QueryState<ProgressOverview> {
  return useNativeQuery('progress', getProgressOverview)
}

export function useVocabularySummaryData(): QueryState<VocabularySummary> {
  return useNativeQuery('vocabulary-summary', getVocabularySummary)
}

export function useLearningMemorySummaryData(): QueryState<LearningMemorySummary> {
  return useNativeQuery('learning-memory-summary', getLearningMemorySummary)
}

export function useStudentLearningSummaryData(): QueryState<StudentLearningSummary> {
  return useNativeQuery('student-learning-summary', getStudentLearningSummary)
}

export function useVocabularyData(filter: VocabularyFilter, search: string, sort: VocabularySort, limit: number, offset: number): QueryState<VocabularyPage> {
  return useNativeQuery(`vocabulary:${filter}:${search}:${sort}:${limit}:${offset}`, () => listVocabularyItems(filter, search, sort, limit, offset))
}

export function useVocabularyItemData(vocabularyId: string): QueryState<VocabularyItemDetails | null> {
  return useNativeQuery(`vocabulary-item:${vocabularyId}`, () => getVocabularyItem(vocabularyId))
}

export function useRecurringMistakesData(limit: number): QueryState<RecurringMistake[]> {
  return useNativeQuery(`recurring-mistakes:${limit}`, () => listRecurringMistakes(limit))
}
