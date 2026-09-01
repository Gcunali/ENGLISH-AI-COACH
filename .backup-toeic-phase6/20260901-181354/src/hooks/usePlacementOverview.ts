import { useCallback, useEffect, useState } from 'react'
import { getPlacementOverview } from '../services/native'
import type { PlacementOverview } from '../types'

export function usePlacementOverview() {
  const [data, setData] = useState<PlacementOverview | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const reload = useCallback(async () => {
    setLoading(true); setError(null)
    try { setData(await getPlacementOverview()) }
    catch (value) { setError(value instanceof Error ? value.message : String(value)) }
    finally { setLoading(false) }
  }, [])
  useEffect(() => { void reload() }, [reload])
  return { data, loading, error, reload }
}
