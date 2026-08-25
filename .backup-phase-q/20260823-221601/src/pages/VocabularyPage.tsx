import { BookOpen, Search } from 'lucide-react'
import { useState } from 'react'
import { Link } from 'react-router-dom'
import { EmptyState, ErrorState, LoadingState } from '../components/PageState'
import { useVocabularyData, useVocabularySummaryData } from '../hooks/useLearningData'
import type { VocabularyFilter, VocabularySort } from '../types'
import { formatLocalDate, humanize } from '../utils/format'

const PAGE_SIZE = 25
const FILTERS: { value: VocabularyFilter; label: string }[] = [
  { value: 'all', label: 'All' }, { value: 'new', label: 'New' },
  { value: 'learning', label: 'Learning' }, { value: 'known', label: 'Known' },
]

export function VocabularyPage() {
  const [filter, setFilter] = useState<VocabularyFilter>('all')
  const [search, setSearch] = useState('')
  const [sort, setSort] = useState<VocabularySort>('recently_seen')
  const [page, setPage] = useState(0)
  const summary = useVocabularySummaryData()
  const query = useVocabularyData(filter, search, sort, PAGE_SIZE, page * PAGE_SIZE)
  const pageCount = query.data ? Math.max(1, Math.ceil(query.data.total / PAGE_SIZE)) : 1

  return <>
    <header className="mb-7"><p className="muted mb-2 text-xs uppercase tracking-[.18em]">Derived from completed analyses</p><h1 className="m-0 text-2xl md:text-3xl">Vocabulary Library</h1><p className="muted text-sm">Words and multiword expressions detected in your analyzed lessons. No new examples are generated.</p></header>
    {summary.loading && <LoadingState label="Loading vocabulary summary…" />}
    {summary.error && <ErrorState message={summary.error} onRetry={summary.reload} />}
    {summary.data && <section className="mb-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-5" aria-label="Vocabulary summary"><SummaryCard label="Total items" value={summary.data.total} /><SummaryCard label="New" value={summary.data.new} /><SummaryCard label="Learning" value={summary.data.learning} /><SummaryCard label="Known" value={summary.data.known} /><SummaryCard label="Contributing lessons" value={summary.data.contributingLessons} /></section>}

    <section className="glass rounded-[28px] p-5 md:p-7">
      <div className="grid gap-4 lg:grid-cols-[1fr_auto]">
        <label className="relative block"><span className="sr-only">Search vocabulary</span><Search size={16} className="muted absolute left-3 top-1/2 -translate-y-1/2" /><input value={search} onChange={(event) => { setSearch(event.target.value); setPage(0) }} placeholder="Search word, phrase, or meaning" className="w-full rounded-xl border border-white/10 bg-[#0d1119] py-3 pl-10 pr-3 text-sm text-white" /></label>
        <label className="muted text-xs">Sort <select aria-label="Sort vocabulary" value={sort} onChange={(event) => { setSort(event.target.value as VocabularySort); setPage(0) }} className="ml-2 rounded-xl border border-white/10 bg-[#0d1119] px-3 py-3 text-white"><option value="recently_seen">Recently seen</option><option value="first_seen">First seen</option><option value="most_frequent">Most frequent</option><option value="alphabetical">Alphabetical</option></select></label>
      </div>
      <div className="mt-4 flex flex-wrap gap-2" aria-label="Vocabulary filters">{FILTERS.map((item) => <button type="button" key={item.value} aria-pressed={filter === item.value} onClick={() => { setFilter(item.value); setPage(0) }} className={`rounded-full border px-4 py-2 text-xs ${filter === item.value ? 'border-[var(--accent)] bg-[var(--accent)]/10 text-[var(--accent)]' : 'border-white/10 bg-white/[.03] text-white'}`}>{item.label}</button>)}</div>

      {query.loading && <div className="mt-5"><LoadingState label="Loading vocabulary from this computer…" /></div>}
      {query.error && <div className="mt-5"><ErrorState message={query.error} onRetry={query.reload} /></div>}
      {query.data?.items.length === 0 && <div className="mt-5"><EmptyState title={search || filter !== 'all' ? 'No matching vocabulary' : 'Your vocabulary will appear here after analyzed lessons.'} message={search || filter !== 'all' ? 'Try another search or status filter.' : 'Complete a lesson analysis to add its structured vocabulary locally.'} /></div>}
      {query.data && query.data.items.length > 0 && <div className="mt-5 grid gap-3 md:grid-cols-2">{query.data.items.map((item) => <Link key={item.id} to={`/vocabulary/${item.id}`} className="rounded-2xl border border-white/[.07] bg-white/[.025] p-4 text-white no-underline"><div className="flex items-start gap-3"><BookOpen size={17} className="mt-0.5 shrink-0 text-[var(--accent)]" /><div className="min-w-0"><div className="flex flex-wrap items-center gap-2"><strong>{item.text}</strong><span className="rounded-full bg-white/5 px-2 py-0.5 text-[10px]">{humanize(item.status)}</span></div><p className="muted mb-0 mt-2 text-sm">{item.meaning}</p>{item.latestExample && <p className="mb-0 mt-3 text-xs">“{item.latestExample}”</p>}<p className="muted mb-0 mt-3 text-[10px]">{item.lessonCount} lesson{item.lessonCount === 1 ? '' : 's'} · last seen {formatLocalDate(item.lastSeenAt)}</p></div></div></Link>)}</div>}

      {query.data && query.data.total > PAGE_SIZE && <nav aria-label="Vocabulary pagination" className="mt-6 flex items-center justify-between gap-3"><button type="button" disabled={page === 0} onClick={() => setPage((value) => Math.max(0, value - 1))} className="rounded-full border border-white/10 px-4 py-2 text-sm disabled:opacity-40">Previous</button><span className="muted text-xs">Page {page + 1} of {pageCount} · {query.data.total} items</span><button type="button" disabled={page + 1 >= pageCount} onClick={() => setPage((value) => value + 1)} className="rounded-full border border-white/10 px-4 py-2 text-sm disabled:opacity-40">Next</button></nav>}
    </section>
  </>
}

function SummaryCard({ label, value }: { label: string; value: number }) { return <div className="glass rounded-2xl p-4"><div className="muted text-[10px] uppercase tracking-wider">{label}</div><div className="mt-2 text-2xl font-semibold">{value}</div></div> }
