import { AlertTriangle, ChevronDown } from 'lucide-react'
import { useState } from 'react'
import { Link } from 'react-router-dom'
import { useRecurringMistakesData } from '../hooks/useLearningData'
import { getRecurringMistake } from '../services/native'
import type { RecurringMistakeDetails } from '../types'
import { formatLocalDate, humanize } from '../utils/format'
import { ErrorState, LoadingState } from './PageState'

export function RecurringMistakesSection({ limit = 10 }: { limit?: number }) {
  const query = useRecurringMistakesData(limit)
  const [expandedId, setExpandedId] = useState<string | null>(null)
  const [details, setDetails] = useState<RecurringMistakeDetails | null>(null)
  const [detailLoading, setDetailLoading] = useState(false)
  const [detailError, setDetailError] = useState<string | null>(null)

  const toggle = async (mistakeId: string) => {
    if (expandedId === mistakeId) {
      setExpandedId(null)
      setDetails(null)
      setDetailError(null)
      return
    }
    setExpandedId(mistakeId)
    setDetails(null)
    setDetailError(null)
    setDetailLoading(true)
    try {
      setDetails(await getRecurringMistake(mistakeId))
    } catch (reason) {
      setDetailError(reason instanceof Error ? reason.message : String(reason))
    } finally {
      setDetailLoading(false)
    }
  }

  return <section className="glass mt-5 rounded-[28px] p-5 md:p-7" aria-labelledby="recurring-mistakes-heading">
    <div className="flex items-start gap-3">
      <AlertTriangle className="mt-0.5 text-amber-300" size={18} />
      <div><h2 id="recurring-mistakes-heading" className="m-0 text-lg">Recurring Mistakes</h2><p className="muted mb-0 mt-1 text-xs">Only correction patterns confirmed across at least two different lessons.</p></div>
    </div>
    {query.loading && <div className="mt-4"><LoadingState label="Loading confirmed recurring mistakes…" /></div>}
    {query.error && <div className="mt-4"><ErrorState message={query.error} onRetry={query.reload} /></div>}
    {query.data?.length === 0 && <p className="muted mb-0 mt-5 text-sm">No recurring mistakes have been confirmed across multiple lessons yet.</p>}
    {query.data && query.data.length > 0 && <div className="mt-5 space-y-3">{query.data.map((mistake) => {
      const expanded = expandedId === mistake.id
      return <article key={mistake.id} className="rounded-2xl border border-white/[.07] bg-white/[.025]">
        <button type="button" aria-expanded={expanded} onClick={() => void toggle(mistake.id)} className="flex w-full items-start gap-3 border-0 bg-transparent p-4 text-left text-white">
          <div className="min-w-0 flex-1"><div className="flex flex-wrap items-center gap-2"><strong className="text-sm">{mistake.title}</strong><span className="rounded-full bg-amber-300/10 px-2 py-0.5 text-[10px] text-amber-200">{humanize(mistake.category)}</span><span className="rounded-full bg-white/5 px-2 py-0.5 text-[10px]">{humanize(mistake.status)}</span></div><p className="muted mb-0 mt-2 text-xs leading-5">{mistake.explanation}</p><p className="muted mb-0 mt-2 text-[11px]">Detected across {mistake.lessonCount} lessons · {mistake.occurrenceCount} occurrences · {formatLocalDate(mistake.firstSeenAt)} to {formatLocalDate(mistake.lastSeenAt)}</p></div>
          <ChevronDown size={17} className={`shrink-0 transition-transform ${expanded ? 'rotate-180' : ''}`} />
        </button>
        {expanded && <div className="border-t border-white/[.07] p-4">
          {detailLoading && <p className="muted text-xs">Loading real occurrences…</p>}
          {detailError && <p role="alert" className="text-xs text-red-200">{detailError}</p>}
          {details && <div className="space-y-3">{details.occurrences.map((occurrence, index) => <div key={`${occurrence.lessonId}-${index}`} className="rounded-xl bg-black/15 p-4 text-sm"><div className="mb-3 flex flex-wrap items-center justify-between gap-2"><span className="muted text-xs">{formatLocalDate(occurrence.lessonDate)}</span><Link to={`/history/${occurrence.lessonId}`} className="text-xs text-white">View lesson</Link></div><p className="mb-2"><span className="muted text-xs">YOU SAID</span><br />{occurrence.original}</p><p className="mb-2"><span className="muted text-xs">BETTER</span><br /><span className="text-[var(--accent)]">{occurrence.corrected}</span></p><p className="muted mb-0 text-xs leading-5">{occurrence.explanation}</p></div>)}</div>}
        </div>}
      </article>
    })}</div>}
  </section>
}
