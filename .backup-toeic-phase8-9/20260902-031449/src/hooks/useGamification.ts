import { useCallback, useEffect, useState } from 'react'
import { getGamificationOverview, listAchievements } from '../services/native'
import type { Achievement, GamificationOverview } from '../types'
import { GAMIFICATION_DATA_CHANGED_EVENT } from '../utils/gamificationData'

function useGamificationQuery<T>(loader: () => Promise<T>) {
  const [data, setData] = useState<T | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [revision, setRevision] = useState(0)
  const reload = useCallback(() => setRevision((value) => value + 1), [])
  useEffect(() => {
    let disposed = false
    setLoading(true); setError(null)
    void loader().then((value) => { if (!disposed) setData(value) })
      .catch((reason: unknown) => { if (!disposed) setError(reason instanceof Error ? reason.message : String(reason)) })
      .finally(() => { if (!disposed) setLoading(false) })
    return () => { disposed = true }
  }, [loader, revision])
  useEffect(() => {
    window.addEventListener(GAMIFICATION_DATA_CHANGED_EVENT, reload)
    return () => window.removeEventListener(GAMIFICATION_DATA_CHANGED_EVENT, reload)
  }, [reload])
  return { data, loading, error, reload }
}

export function useGamificationOverview() { return useGamificationQuery<GamificationOverview>(getGamificationOverview) }
export function useAchievements() { return useGamificationQuery<Achievement[]>(listAchievements) }
