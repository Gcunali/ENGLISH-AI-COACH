import { useState } from 'react'
import { Link } from 'react-router-dom'
import { EmptyState, ErrorState, LoadingState } from '../components/PageState'
import { ProgressChart } from '../components/ProgressChart'
import { useProgressData } from '../hooks/useLearningData'
import type { ScoreDimension } from '../types'
import { formatDuration, formatLocalDate, humanize } from '../utils/format'

const DIMENSIONS: { value: ScoreDimension; label: string }[] = [
  { value: 'fluency', label: 'Fluency' }, { value: 'grammar', label: 'Grammar' },
  { value: 'vocabulary', label: 'Vocabulary' }, { value: 'comprehension', label: 'Comprehension' },
  { value: 'interaction', label: 'Interaction' },
]

export function ProgressPage() {
  const query = useProgressData()
  const [dimension, setDimension] = useState<ScoreDimension>('grammar')
  if (query.loading) return <LoadingState label="Loading progress from completed local analyses…" />
  if (query.error) return <ErrorState message={query.error} onRetry={query.reload} />
  if (!query.data || query.data.analyzedLessonCount === 0 || !query.data.averages) return <><PageHeader /><EmptyState title="No analyzed lessons yet" message="Complete an analyzed lesson to start tracking progress." /></>
  const progress = query.data
  const averages = progress.averages!
  const onePoint = progress.analyzedLessonCount === 1

  return <>
    <PageHeader />
    <section className="mb-5 grid gap-3 sm:grid-cols-2 xl:grid-cols-6" aria-label="Average performance">
      <AverageCard label="Overall" value={averages.overall} />
      {DIMENSIONS.map((item) => <AverageCard key={item.value} label={item.label} value={averages[item.value]} />)}
    </section>
    <section className="mb-5 grid gap-4 md:grid-cols-2"><Insight label="Strongest area" value={progress.strongestAreas.map(humanize).join(' · ')} detail="Highest factual average; ties are preserved." /><Insight label="Focus area" value={progress.focusAreas.map(humanize).join(' · ')} detail="Lowest factual average; not an LLM judgment." /></section>
    {onePoint && <div role="note" className="mb-5 rounded-2xl border border-sky-300/15 bg-sky-300/[.06] p-4 text-sm">One analyzed lesson is available. Complete more analyzed lessons to see your progress over time; no trend is inferred from this point.</div>}
    <div className="grid gap-5 xl:grid-cols-2">
      <ProgressChart title="Overall score over time" points={progress.points.map((point, index) => ({ label: `Lesson ${index + 1} · ${formatLocalDate(point.date)}`, value: point.overall }))} />
      <div><div className="mb-3 flex flex-wrap items-center justify-between gap-3"><h2 className="m-0 text-base">Score by dimension</h2><label className="muted text-xs">Dimension <select value={dimension} onChange={(event) => setDimension(event.target.value as ScoreDimension)} className="ml-2 rounded-lg border border-white/10 bg-[#0d1119] px-3 py-2 text-white">{DIMENSIONS.map((item) => <option key={item.value} value={item.value}>{item.label}</option>)}</select></label></div><ProgressChart title={`${humanize(dimension)} over time`} points={progress.points.map((point, index) => ({ label: `Lesson ${index + 1} · ${formatLocalDate(point.date)}`, value: point[dimension] }))} /></div>
    </div>
    {progress.latestRecommendation && <section className="glass mt-5 rounded-2xl p-5"><p className="muted mb-1 text-[10px] uppercase tracking-widest">Suggested next focus · latest analysis</p><p className="mb-0 text-sm">{progress.latestRecommendation}</p></section>}
    <section className="glass mt-5 rounded-2xl p-5"><h2 className="mt-0 text-base">Recent analyzed lessons</h2><div className="space-y-2">{[...progress.points].reverse().slice(0, 5).map((point) => <div key={point.lessonId} className="flex flex-wrap items-center justify-between gap-3 rounded-xl bg-white/[.035] p-3 text-sm"><div><div>{formatLocalDate(point.date)}</div><div className="muted text-xs">{formatDuration(point.durationSeconds)}</div></div><div className="flex items-center gap-4"><strong className="text-[var(--accent)]">{point.overall}</strong><Link to={`/history/${point.lessonId}`} className="text-white">Details</Link></div></div>)}</div></section>
    <p className="muted text-xs">Pronunciation is not included because acoustic analysis is not available. Scores are local pedagogical indicators, not official certification results.</p>
  </>
}

function PageHeader() { return <header className="mb-7"><p className="muted text-xs uppercase tracking-[.18em] mb-2">Persisted analyses only</p><h1 className="m-0 text-2xl md:text-3xl">Progress</h1><p className="muted text-sm">Every point represents one completed analysis. Missing lessons are never treated as zero.</p></header> }
function AverageCard({ label, value }: { label: string; value: number }) { return <div className="glass rounded-2xl p-4"><div className="muted text-[10px] uppercase tracking-wider">Average {label}</div><div className="mt-2 text-2xl font-semibold">{value}</div></div> }
function Insight({ label, value, detail }: { label: string; value: string; detail: string }) { return <div className="glass rounded-2xl p-5"><div className="muted text-[10px] uppercase tracking-wider">{label}</div><div className="mt-2 text-xl font-semibold text-[var(--accent)]">{value}</div><div className="muted mt-1 text-xs">{detail}</div></div> }
