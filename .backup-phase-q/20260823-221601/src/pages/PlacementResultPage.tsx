import { useEffect, useState } from 'react'
import { Link, useParams } from 'react-router-dom'
import { ErrorState, LoadingState } from '../components/PageState'
import { getPlacementResult } from '../services/native'
import type { PlacementResult } from '../types'
import { formatLocalDate, humanize } from '../utils/format'

export function PlacementResultPage() {
  const { attemptId = '' } = useParams()
  const [result, setResult] = useState<PlacementResult | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  useEffect(() => { setLoading(true); getPlacementResult(attemptId).then((value) => { if (!value) throw new Error('Completed placement result not found.'); setResult(value) }).catch((value) => setError(value instanceof Error ? value.message : String(value))).finally(() => setLoading(false)) }, [attemptId])
  if (loading) return <LoadingState label="Loading placement result…" />
  if (error || !result) return <ErrorState message={error ?? 'Placement result not found.'} onRetry={() => window.location.reload()} />
  return <><header className="mb-7"><p className="muted text-xs uppercase tracking-[.18em] mb-2">Completed {formatLocalDate(result.attempt.completedAt!)}</p><h1 className="m-0 text-2xl md:text-3xl">Estimated CEFR Level</h1></header>
    <section className="glass rounded-[28px] p-7 text-center"><div className="text-6xl font-semibold text-[var(--accent)]">{result.estimatedCefrLevel}</div><p className="mt-3 text-lg">{humanize(result.confidence)} confidence</p><p className="muted mx-auto max-w-2xl text-sm">{result.disclaimer}</p></section>
    <section className="mt-5 grid gap-3 sm:grid-cols-2 xl:grid-cols-4" aria-label="Placement domain results">{result.domains.map((domain) => <div key={domain.skill} className="glass rounded-2xl p-5"><p className="muted mt-0 text-xs uppercase tracking-wider">{humanize(domain.skill)}</p><div className="text-2xl font-semibold">{domain.assessed ? domain.level : 'Not assessed'}</div></div>)}</section>
    <section className="glass mt-5 rounded-2xl p-5"><h2 className="text-base">Skills not assessed</h2><div className="grid gap-3 sm:grid-cols-3">{['Listening','Pronunciation','Formal writing'].map((skill) => <div key={skill} className="rounded-xl bg-white/[.035] p-3"><strong className="text-sm">{skill}</strong><div className="muted text-xs">Not assessed</div></div>)}</div></section>
    {result.speakingSummary && <section className="glass mt-5 rounded-2xl p-5"><h2 className="text-base">Spoken Production summary</h2><p className="text-sm">{result.speakingSummary}</p><div className="space-y-3">{result.speakingEvidence.map((item, index) => <div key={`${item.criterion}-${index}`} className="rounded-xl bg-white/[.035] p-4 text-sm"><strong>{humanize(item.criterion)}</strong><p className="muted">{item.observation}</p><q>{item.example}</q></div>)}</div><p className="muted mb-0 text-xs">Evidence is transcript-based. Pronunciation was not evaluated.</p></section>}
    <div className="mt-6 flex flex-wrap gap-3"><Link to="/placement" className="rounded-full bg-[var(--accent)] px-5 py-3 font-semibold text-black no-underline">Retake placement</Link><Link to="/progress" className="rounded-full border border-white/15 px-5 py-3 text-white no-underline">View Progress</Link></div>
  </>
}
