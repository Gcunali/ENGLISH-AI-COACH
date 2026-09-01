import { AlertTriangle, Headphones, MessageSquareText, Mic2, Play, Sparkles } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { ErrorState, LoadingState } from '../components/PageState'
import { AppCard, InlineNotice, PageHero, PageShell, SectionHeader } from '../components/ProductUI'
import { getPracticeAvailability, startPracticeSession } from '../services/native'
import type { PracticeAvailability, PracticeMode } from '../types'

const MODES: Array<{ mode: PracticeMode; title: string; description: string; icon: typeof Sparkles; count: keyof PracticeAvailability }> = [
  { mode: 'daily', title: 'Daily Practice', description: 'A short, rotating mix built from material you have really studied.', icon: Sparkles, count: 'dailyCount' },
  { mode: 'dictation', title: 'Dictation', description: 'Listen without seeing the answer, type what you hear, then compare word by word.', icon: Headphones, count: 'dictationCount' },
  { mode: 'shadowing', title: 'Shadowing', description: 'Listen first, repeat the whole phrase, and receive local acoustic feedback.', icon: Mic2, count: 'shadowingCount' },
  { mode: 'speaking_recall', title: 'Speaking Recall', description: 'Respond to a familiar situation before revealing a useful model expression.', icon: MessageSquareText, count: 'speakingRecallCount' },
  { mode: 'mistake_repair', title: 'Mistake Repair', description: 'Recognize, rebuild, and say corrections confirmed across multiple lessons.', icon: AlertTriangle, count: 'mistakeRepairCount' },
]

export function PracticePage() {
  const navigate = useNavigate()
  const [data, setData] = useState<PracticeAvailability | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [busy, setBusy] = useState<PracticeMode | null>(null)
  const load = () => { setError(null); void getPracticeAvailability().then(setData).catch(value => setError(message(value))) }
  useEffect(load, [])
  if (error && !data) return <ErrorState message={error} onRetry={load} />
  if (!data) return <LoadingState label="Building your local practice options…" />
  const start = async (mode: PracticeMode) => {
    setBusy(mode); setError(null)
    try { const session = await startPracticeSession(mode, mode === 'daily' ? 7 : 5); navigate(`/practice/session/${session.id}`) }
    catch (value) { setError(message(value)) }
    finally { setBusy(null) }
  }
  return <PageShell width="wide">
    <PageHero eyebrow="Offline learning lab" title="Practice the skills" accent="you want to keep." description="Every queue is deterministic and comes from your own local lesson history—never from invented progress." compact />
    {error && <InlineNotice tone="warning" live>{error}</InlineNotice>}
    <AppCard className="mt-4">
      <SectionHeader title="Choose a practice mode" description="Daily Practice mixes eligible items. Focus modes let you work on one skill at a time." />
      <div className="practice-mode-grid mt-6">{MODES.map(entry => {
        const Icon = entry.icon; const available = Number(data[entry.count])
        return <article key={entry.mode} className="practice-mode-card">
          <div className="lesson-mode-icon"><Icon aria-hidden="true" /></div>
          <div className="min-w-0 flex-1"><h3>{entry.title}</h3><p>{entry.description}</p><span>{available} eligible item{available === 1 ? '' : 's'}</span></div>
          <button className="button-primary" disabled={available === 0 || busy !== null} onClick={() => void start(entry.mode)}><Play size={16} fill="currentColor" />{busy === entry.mode ? 'Starting…' : 'Start'}</button>
        </article>
      })}</div>
    </AppCard>
    {data.confirmedMistakeCount === 0 && <p className="page-footnote"><AlertTriangle size={13} /> Mistake Repair is intentionally empty until the same pattern is confirmed in at least two lessons.</p>}
  </PageShell>
}
function message(value: unknown) { return value instanceof Error ? value.message : String(value) }
