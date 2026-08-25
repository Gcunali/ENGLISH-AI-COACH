import { ArrowLeft, BookOpen } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Link, useParams } from 'react-router-dom'
import { EmptyState, ErrorState, LoadingState } from '../components/PageState'
import { useVocabularyItemData } from '../hooks/useLearningData'
import { updateVocabularyStatus } from '../services/native'
import type { VocabularyStatus } from '../types'
import { formatLocalDate, humanize } from '../utils/format'
import { notifyLearningDataChanged } from '../utils/learningData'

export function VocabularyDetailsPage() {
  const { vocabularyId = '' } = useParams()
  const query = useVocabularyItemData(vocabularyId)
  const [status, setStatus] = useState<VocabularyStatus>('new')
  const [saving, setSaving] = useState(false)
  const [saveError, setSaveError] = useState<string | null>(null)
  useEffect(() => { if (query.data) setStatus(query.data.item.status) }, [query.data])

  const changeStatus = async (next: VocabularyStatus) => {
    const previous = status
    setStatus(next)
    setSaving(true)
    setSaveError(null)
    try {
      await updateVocabularyStatus(vocabularyId, next)
      notifyLearningDataChanged()
      query.reload()
    } catch (reason) {
      setStatus(previous)
      setSaveError(reason instanceof Error ? reason.message : String(reason))
    } finally {
      setSaving(false)
    }
  }

  if (query.loading) return <LoadingState label="Loading vocabulary details…" />
  if (query.error) return <ErrorState message={query.error} onRetry={query.reload} />
  if (!query.data) return <EmptyState title="Vocabulary item not found" message="This item does not exist in the local learning memory." />
  const { item, occurrences } = query.data
  return <>
    <Link to="/vocabulary" className="muted mb-5 inline-flex items-center gap-2 text-sm no-underline"><ArrowLeft size={15} /> Back to Vocabulary</Link>
    <header className="glass rounded-[28px] p-5 md:p-7"><div className="flex flex-wrap items-start justify-between gap-5"><div className="flex items-start gap-3"><BookOpen className="mt-1 text-[var(--accent)]" size={20} /><div><p className="muted mb-1 text-[10px] uppercase tracking-widest">Vocabulary item</p><h1 className="m-0 text-2xl md:text-3xl">{item.text}</h1><p className="muted mb-0 mt-3 text-sm">{item.meaning}</p></div></div><label className="text-xs">Learning status <select aria-label="Learning status" value={status} disabled={saving} onChange={(event) => void changeStatus(event.target.value as VocabularyStatus)} className="ml-2 rounded-xl border border-white/10 bg-[#0d1119] px-3 py-2 text-white"><option value="new">New</option><option value="learning">Learning</option><option value="known">Known</option></select></label></div>{saveError && <p role="alert" className="mt-4 text-sm text-red-200">{saveError}</p>}<div className="mt-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-4"><Detail label="Status" value={humanize(status)} /><Detail label="First seen" value={formatLocalDate(item.firstSeenAt)} /><Detail label="Last seen" value={formatLocalDate(item.lastSeenAt)} /><Detail label="Lessons" value={String(item.lessonCount)} /></div></header>
    <section className="glass mt-5 rounded-[28px] p-5 md:p-7"><h2 className="mt-0 text-lg">Examples from analyzed lessons</h2><div className="space-y-3">{occurrences.map((occurrence) => <article key={occurrence.lessonId} className="rounded-2xl border border-white/[.07] bg-white/[.025] p-4"><div className="flex flex-wrap items-center justify-between gap-3"><span className="muted text-xs">{formatLocalDate(occurrence.lessonDate)}</span><Link to={`/history/${occurrence.lessonId}`} className="text-xs text-white">View lesson</Link></div><p className="mb-0 mt-3 text-sm">“{occurrence.example}”</p>{occurrence.occurrenceCount > 1 && <p className="muted mb-0 mt-2 text-[10px]">{occurrence.occurrenceCount} structured occurrences in this analysis</p>}</article>)}</div></section>
  </>
}

function Detail({ label, value }: { label: string; value: string }) { return <div className="rounded-xl bg-white/[.035] p-3"><div className="font-semibold text-sm">{value}</div><div className="muted mt-1 text-[10px]">{label}</div></div> }
