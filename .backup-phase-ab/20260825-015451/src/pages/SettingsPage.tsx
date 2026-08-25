import { Activity, Brain, Gauge, ShieldCheck, UserRound } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { ErrorState, LoadingState } from '../components/PageState'
import { useStudentLearningSummaryData } from '../hooks/useLearningData'
import { getLearningMemoryEnabled, getStreamingVoiceResponseEnabled, setLearningMemoryEnabled, setStreamingVoiceResponseEnabled } from '../services/native'
import { notifyLearningDataChanged } from '../utils/learningData'
import { DataBackupSection } from '../components/DataBackupSection'
import { InlineNotice, PageHeader, PageShell, SectionHeader, ToggleRow } from '../components/ProductUI'

export function SettingsPage() {
  const summary = useStudentLearningSummaryData()
  const [enabled, setEnabled] = useState<boolean | null>(null)
  const [settingError, setSettingError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)
  const [streamingEnabled, setStreamingEnabled] = useState<boolean | null>(null)
  const [streamingError, setStreamingError] = useState<string | null>(null)
  const [savingStreaming, setSavingStreaming] = useState(false)
  const [savedNotice, setSavedNotice] = useState<string | null>(null)

  useEffect(() => {
    let disposed = false
    void getLearningMemoryEnabled()
      .then((value) => { if (!disposed) setEnabled(value) })
      .catch((reason: unknown) => {
        if (!disposed) setSettingError(reason instanceof Error ? reason.message : String(reason))
      })
    return () => { disposed = true }
  }, [])

  useEffect(() => {
    let disposed = false
    void getStreamingVoiceResponseEnabled()
      .then((value) => { if (!disposed) setStreamingEnabled(value) })
      .catch((reason: unknown) => {
        if (!disposed) setStreamingError(reason instanceof Error ? reason.message : String(reason))
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
    setSavedNotice(null)
    try {
      await setLearningMemoryEnabled(next)
      notifyLearningDataChanged()
      setSavedNotice('Learning preference saved locally.')
    } catch (reason) {
      setEnabled(previous)
      setSettingError(reason instanceof Error ? reason.message : String(reason))
    } finally {
      setSaving(false)
    }
  }

  async function toggleStreaming() {
    if (streamingEnabled === null || savingStreaming) return
    const previous = streamingEnabled
    const next = !previous
    setStreamingEnabled(next)
    setSavingStreaming(true)
    setStreamingError(null)
    setSavedNotice(null)
    try {
      await setStreamingVoiceResponseEnabled(next)
      setSavedNotice('Voice preference saved locally.')
    } catch (reason) {
      setStreamingEnabled(previous)
      setStreamingError(reason instanceof Error ? reason.message : String(reason))
    } finally {
      setSavingStreaming(false)
    }
  }

  return <PageShell width="standard">
    <PageHeader eyebrow="Local preferences" title="Settings" description="Manage voice behavior, learning context, local data and system tools." />
    <nav aria-label="Settings sections" className="mb-5 flex flex-wrap gap-2"><a className="button-ghost" href="#settings-voice">Voice</a><a className="button-ghost" href="#settings-learning">Learning</a><a className="button-ghost" href="#settings-data">Privacy &amp; Data</a><a className="button-ghost" href="#settings-system">System</a></nav>
    {savedNotice && <div className="mb-5"><InlineNotice tone="success" live>{savedNotice}</InlineNotice></div>}

    <section id="settings-voice" className="glass scroll-mt-5 rounded-[28px] p-5 md:p-7" aria-labelledby="voice-performance-title">
      <SectionHeader id="voice-performance-title" title="Voice" description="Conversation response preferences. Calibrated microphone and model defaults remain automatic." />
      <div className="mt-5"><ToggleRow label="Faster voice responses" description="Start speaking while the local AI is still finishing its response." checked={streamingEnabled} busy={savingStreaming} onChange={()=>void toggleStreaming()} icon={<Gauge size={21}/>} /></div>
      {streamingError && <p role="alert" className="mt-4 text-sm text-red-300">{streamingError}</p>}
    </section>

    <section id="settings-learning" className="glass mt-5 scroll-mt-5 rounded-[28px] p-5 md:p-7">
      <SectionHeader title="Learning" description="Choose how confirmed learning history supports future conversations." actions={<Link to="/profile" className="button-secondary"><UserRound size={16}/>Student Profile</Link>} />
      <div className="mt-5"><ToggleRow label="Use learning memory in lessons" description="Give the local teacher a compact snapshot of confirmed learning priorities when a lesson starts." checked={enabled} busy={saving} onChange={()=>void toggleMemory()} icon={<Brain size={21}/>} /></div>
      {settingError && <p role="alert" className="mt-4 text-sm text-red-300">{settingError}</p>}
    </section>

    <section className="glass mt-5 rounded-[28px] p-5 md:p-7" aria-labelledby="teacher-memory-title">
      <SectionHeader id="teacher-memory-title" title="What the teacher remembers" description="A read-only preview of the compact teaching context." />
      <div className="mt-5">
      {summary.loading && <LoadingState label="Loading your local learning summary…" />}
      {summary.error && <ErrorState message={summary.error} onRetry={summary.reload} />}
      {summary.data && <MemorySummary summary={summary.data} />}
      </div>
    </section>

    <div id="settings-data" className="scroll-mt-5"><DataBackupSection /></div>
    <div className="mt-5"><InlineNotice tone="info"><span className="inline-flex gap-2"><ShieldCheck aria-hidden="true" size={18}/>Learning data and backups stay on this computer. Backup files are not encrypted, so keep them in a trusted location.</span></InlineNotice></div>
    <section id="settings-system" className="glass mt-5 scroll-mt-5 rounded-[28px] p-5 md:p-7"><SectionHeader title="System" description="Inspect local component readiness and copy a privacy-safe diagnostic report." actions={<Link to="/diagnostics" className="button-secondary"><Activity size={16}/>Open Diagnostics</Link>} /></section>
  </PageShell>
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
