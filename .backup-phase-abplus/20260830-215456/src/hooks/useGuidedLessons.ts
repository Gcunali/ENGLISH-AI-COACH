import { useCallback, useEffect, useState } from 'react'
import { getGuidedLessonOverview, listGuidedLessons, listRecentGuidedLessonSessions } from '../services/native'
import type { GuidedLessonOverview, GuidedLessonSession, GuidedLessonSummary } from '../types'

export function useGuidedLessonLibrary(){
  const[data,setData]=useState<{overview:GuidedLessonOverview;lessons:GuidedLessonSummary[];recent:GuidedLessonSession[]}|null>(null)
  const[loading,setLoading]=useState(true);const[error,setError]=useState<string|null>(null)
  const reload=useCallback(async()=>{setLoading(true);setError(null);try{const[overview,lessons,recent]=await Promise.all([getGuidedLessonOverview(),listGuidedLessons(),listRecentGuidedLessonSessions(10)]);setData({overview,lessons,recent})}catch(value){setError(value instanceof Error?value.message:String(value))}finally{setLoading(false)}},[])
  useEffect(()=>{void reload()},[reload]);return{data,loading,error,reload}
}
