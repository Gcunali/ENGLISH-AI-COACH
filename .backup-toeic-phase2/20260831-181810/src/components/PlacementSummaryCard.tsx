import { Link } from 'react-router-dom'
import { usePlacementOverview } from '../hooks/usePlacementOverview'
import { formatLocalDate, humanize } from '../utils/format'

export function PlacementSummaryCard({ compact = false }: { compact?: boolean }) {
  const query = usePlacementOverview()
  if (query.loading) return <section className="glass rounded-2xl p-5" aria-label="Placement"><p className="muted m-0 text-sm">Loading placement estimate…</p></section>
  if (query.error) return <section className="glass rounded-2xl p-5" aria-label="Placement"><p className="muted m-0 text-sm">Placement estimate unavailable.</p></section>
  const result = query.data?.currentResult
  return <section className="glass rounded-2xl p-5" aria-label="Estimated English Level"><p className="muted mt-0 text-[10px] uppercase tracking-widest">Estimated English Level</p>{result ? <div className={compact ? 'flex flex-wrap items-end justify-between gap-3' : ''}><div><div className="text-3xl font-semibold text-[var(--accent)]">{result.estimatedCefrLevel}</div><p className="muted mb-0 text-xs">Confidence: {humanize(result.confidence)} · {formatLocalDate(result.attempt.completedAt!)}</p></div><Link to={`/placement/results/${result.attempt.id}`} className="text-sm text-white">View result</Link></div> : <><p className="text-sm">No placement estimate yet.</p><Link to="/placement" className="text-sm text-white">Take Placement Test</Link></>}</section>
}
