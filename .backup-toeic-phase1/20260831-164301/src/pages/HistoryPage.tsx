import { AlertTriangle, CheckCircle2, ChevronLeft, ChevronRight, FileQuestion } from 'lucide-react'
import { useState } from 'react'
import { Link } from 'react-router-dom'
import { EmptyState, ErrorState, LoadingState } from '../components/PageState'
import { PageHeader, PageShell } from '../components/ProductUI'
import { useHistoryData } from '../hooks/useLearningData'
import type { LessonHistoryFilter, LessonHistoryItem } from '../types'
import { formatDuration, formatLocalDate, humanize, lessonTitle } from '../utils/format'

const PAGE_SIZE = 20
const FILTERS: { value: LessonHistoryFilter; label: string }[] = [
  { value: 'all', label: 'All' }, { value: 'completed', label: 'Completed' },
  { value: 'interrupted', label: 'Interrupted' }, { value: 'analyzed', label: 'With analysis' },
  { value: 'unanalyzed', label: 'Without analysis' },
]

export function HistoryPage() {
  const [filter, setFilter] = useState<LessonHistoryFilter>('all')
  const [offset, setOffset] = useState(0)
  const query = useHistoryData(filter, PAGE_SIZE, offset)
  const page = Math.floor(offset / PAGE_SIZE) + 1
  const pageCount = query.data ? Math.max(1, Math.ceil(query.data.total / PAGE_SIZE)) : 1

  const chooseFilter = (next: LessonHistoryFilter) => {
    setFilter(next)
    setOffset(0)
  }

  return <PageShell width="standard">
    <PageHeader eyebrow="Local lesson archive" title="History" description="Completed and interrupted lessons, newest first. Opening this page never calls the analyzer." actions={<Link to="/lesson/new" className="button-primary">Start a Lesson</Link>} />
    <div className="mb-5 flex flex-wrap gap-2" aria-label="History filters">
      {FILTERS.map((item) => <button key={item.value} onClick={() => chooseFilter(item.value)} aria-pressed={filter === item.value} className={`rounded-full border px-4 py-2 text-sm ${filter === item.value ? 'border-[var(--accent)]/40 bg-[var(--accent)]/10 text-[var(--accent)]' : 'border-white/10 bg-white/[.03] text-white'}`}>{item.label}</button>)}
    </div>
    {query.loading && <LoadingState label="Loading lesson history from SQLite…" />}
    {query.error && <ErrorState message={query.error} onRetry={query.reload} />}
    {query.data && query.data.items.length === 0 && <EmptyState title="No lessons found" message={filter === 'all' ? 'Completed lessons will appear here with their transcript and analysis.' : 'No saved lessons match this filter.'} action={filter === 'all' ? <Link to="/lesson/new" className="button-primary">Start a Lesson</Link> : undefined} />}
    {query.data && query.data.items.length > 0 && <div className="space-y-3" aria-label="Lesson history list">
      {query.data.items.map((lesson) => <LessonHistoryCard key={lesson.id} lesson={lesson} />)}
    </div>}
    {query.data && query.data.total > PAGE_SIZE && <nav className="mt-5 flex items-center justify-between" aria-label="History pagination"><button disabled={offset === 0} onClick={() => setOffset(Math.max(0, offset - PAGE_SIZE))} className="flex items-center gap-2 rounded-full border border-white/10 bg-white/[.03] px-4 py-2 text-sm text-white disabled:opacity-40"><ChevronLeft size={15} /> Previous</button><span className="muted text-sm">Page {page} of {pageCount}</span><button disabled={offset + PAGE_SIZE >= query.data.total} onClick={() => setOffset(offset + PAGE_SIZE)} className="flex items-center gap-2 rounded-full border border-white/10 bg-white/[.03] px-4 py-2 text-sm text-white disabled:opacity-40">Next <ChevronRight size={15} /></button></nav>}
  </PageShell>
}

function LessonHistoryCard({ lesson }: { lesson: LessonHistoryItem }) {
  const StatusIcon = lesson.status === 'completed' ? CheckCircle2 : lesson.status === 'interrupted' ? AlertTriangle : FileQuestion
  return <article className="glass rounded-2xl p-5"><div className="flex flex-wrap items-center justify-between gap-4"><div className="min-w-0 flex-1"><div className="flex items-start gap-2"><StatusIcon aria-hidden="true" size={16} className={`mt-1 shrink-0 ${lesson.status === 'completed' ? 'text-[var(--accent)]' : 'text-amber-300'}`} /><h2 className="m-0 break-words text-base">{lesson.customTitle || lessonTitle(lesson.topic) || lesson.modeTitle}</h2></div><p className="muted mb-0 mt-2 text-xs">{formatLocalDate(lesson.startedAt)} · {formatDuration(lesson.durationSeconds)} · {lesson.modeTitle}</p><div className="mt-3 flex flex-wrap gap-2 text-[11px]"><Badge label={`Lesson: ${humanize(lesson.status)}`} /><Badge label={`${lesson.studentTurnCount} student turns`} /><Badge label={`${lesson.correctionCount} corrections`} /><Badge label={`Analysis: ${lesson.analysisStatus ? humanize(lesson.analysisStatus) : 'Not analyzed'}`} /></div></div><div className="flex flex-wrap items-center gap-4">{lesson.overallScore !== null && <div className="text-center"><strong className="text-2xl text-[var(--accent)]">{lesson.overallScore}</strong><div className="muted text-[10px]">Overall</div></div>}<Link to={`/history/${lesson.id}`} className="button-secondary">View details</Link></div></div></article>
}

function Badge({ label }: { label: string }) { return <span className="rounded-full border border-white/[.07] bg-white/[.035] px-2.5 py-1">{label}</span> }
