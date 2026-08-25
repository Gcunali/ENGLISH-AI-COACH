import { Brain, ShieldCheck } from 'lucide-react'
import { useEffect, useState } from 'react'
import { ErrorState, LoadingState } from '../components/PageState'
import { useStudentLearningSummaryData } from '../hooks/useLearningData'
import { getLearningMemoryEnabled, setLearningMemoryEnabled } from '../services/native'
import { notifyLearningDataChanged } from '../utils/learningData'

export function SettingsPage() {
  const summary = useStudentLearningSummaryData()
  const [enabled, setEnabled] = useState<boolean | null>(null)
  const [settingError, setSettingError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    let disposed = false
    void getLearningMemoryEnabled()
      .then((value) => { if (!disposed) setEnabled(value) })
      .catch((reason: unknown) => {
        if (!disposed) setSettingError(reason instanceof Error ? reason.message : String(reason))
      })
    return () => { disposed = true }
  }, [])

  async function toggleMemory() {
    if (enabled === null || saving) return
    const previous = enabled
    const next = !previous
    setEnabled(next)
    setSaving(true)
    setSettingError(null)
    try {
      await setLearningMemoryEnabled(next)
      notifyLearningDataChanged()
    } catch (reason) {
      setEnabled(previous)
      setSettingError(reason instanceof Error ? reason.message : String(reason))
    } finally {
      setSaving(false)
    }
  }

  return <>
    <header className="mb-7">
      <p className="muted mb-2 text-xs uppercase tracking-[.18em]">Local learning memory</p>
      <h1 className="m-0 text-2xl md:text-3xl">Settings</h1>
      <p className="muted text-sm">Control how your persisted learning history supports future lessons.</p>
    </header>

    <section className="glass rounded-[28px] p-5 md:p-7">
      <div className="flex flex-wrap items-start justify-between gap-5">
        <div className="min-w-0 flex-1 gap-3 flex">
          <Brain className="mt-1 shrink-0 text-[var(--accent)]" size={21} />
          <div>
            <h2 className="m-0 text-lg">Use learning memory in lessons</h2>
            <p className="muted mb-0 mt-2 text-sm">When enabled, the local teacher receives a compact snapshot of confirmed learning priorities when a lesson starts.</p>
          </div>
        </div>
        <button
          type="button"
          role="switch"
          aria-checked={enabled ?? false}
          disabled={enabled === null || saving}
          onClick={toggleMemory}
          className={`min-w-24 shrink-0 rounded-full border px-4 py-2 text-sm ${enabled ? 'border-[var(--accent)] bg-[var(--accent)]/10 text-[var(--accent)]' : 'border-white/10 bg-white/[.03] text-white'} disabled:opacity-50`}
        >{enabled ? 'On' : 'Off'}</button>
      </div>
      {settingError && <p role="alert" className="mt-4 text-sm text-red-300">{settingError}</p>}
    </section>

    <section className="glass mt-5 rounded-[28px] p-5 md:p-7">
      <h2 className="mt-0 text-lg">What the teacher remembers</h2>
      {summary.loading && <LoadingState label="Loading your local learning summary…" />}
      {summary.error && <ErrorState message={summary.error} onRetry={summary.reload} />}
      {summary.data && <MemorySummary summary={summary.data} />}
    </section>

    <section className="mt-5 flex gap-3 rounded-2xl border border-sky-300/15 bg-sky-300/[.05] p-4">
      <ShieldCheck className="mt-0.5 shrink-0 text-sky-200" size={19} />
      <p className="muted m-0 text-sm">This summary stays on this computer. It is derived from completed local analyses and confirmed learning records—not raw lesson transcripts or personal facts.</p>
    </section>
  </>
}

function MemorySummary({ summary }: { summary: import('../types').StudentLearningSummary }) {
  const empty = summary.analyzedLessonCount === 0
  if (empty) return <div className="rounded-2xl bg-white/[.03] p-5"><p className="m-0 font-medium">No learning memory yet</p><p className="muted mb-0 mt-2 text-sm">Complete and analyze lessons to build a compact teaching summary.</p></div>
  return <>
    <p className="muted text-xs">Built from {summary.analyzedLessonCount} analyzed lesson{summary.analyzedLessonCount === 1 ? '' : 's'}.</p>
    <div className="mt-4 grid gap-4 md:grid-cols-2">
      <MemoryList title="Recent strengths" items={summary.recentStrengths.map((item) => item.title)} empty="No strengths recorded yet." />
      <MemoryList title="Current focus" items={summary.currentFocusAreas.map((item) => `${item.area}: ${item.title}`)} empty="No focus areas recorded yet." />
      <MemoryList title="Confirmed recurring mistakes" items={summary.confirmedRecurringMistakes.map((item) => `${item.title} · ${item.lessonCount} lessons`)} empty="No recurring mistakes have been confirmed across multiple lessons." />
      <MemoryList title="Recent vocabulary" items={summary.recentVocabulary.map((item) => `${item.text} — ${item.meaning} (${item.status})`)} empty="No active vocabulary items." />
      <div className="md:col-span-2"><MemoryList title="Next lesson recommendations" items={summary.nextLessonRecommendations} empty="No recommendations recorded yet." /></div>
    </div>
  </>
}

function MemoryList({ title, items, empty }: { title: string; items: string[]; empty: string }) {
  return <div className="rounded-2xl bg-white/[.03] p-4"><h3 className="m-0 text-sm">{title}</h3>{items.length > 0 ? <ul className="muted mb-0 mt-3 space-y-2 pl-5 text-sm">{items.map((item) => <li key={item}>{item}</li>)}</ul> : <p className="muted mb-0 mt-3 text-sm">{empty}</p>}</div>
}
