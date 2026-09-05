import { AlertTriangle, BookOpen, Brain, Clock3, History, Info, Play, RotateCcw, Shuffle, Target } from 'lucide-react'
import { useState } from 'react'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { ConfirmationDialog } from '../components/ConfirmationDialog'
import { ErrorState, LoadingState } from '../components/PageState'
import { AppCard, PageHero, PageShell, SectionHeader } from '../components/ProductUI'
import { useReviewOverview } from '../hooks/useReviewData'
import { startReviewSession } from '../services/native'
import type { ReviewMode } from '../types'
import { formatLocalDate, humanize } from '../utils/format'

const REVIEW_MODES: Array<{ value: ReviewMode; label: string; description: string; icon: typeof Shuffle }> = [
  { value: 'mixed', label: 'Mixed Review', description: 'A balanced mix of vocabulary and recurring mistakes.', icon: Shuffle },
  { value: 'vocabulary', label: 'Vocabulary', description: 'Focus only on words and phrases.', icon: BookOpen },
  { value: 'mistakes', label: 'Recurring Mistakes', description: 'Target your confirmed mistakes.', icon: AlertTriangle },
]

export function ReviewPage() {
  const query = useReviewOverview()
  const navigate = useNavigate()
  const [params] = useSearchParams()
  const initial = params.get('mode')
  const [mode, setMode] = useState<ReviewMode>(initial === 'vocabulary' || initial === 'mistakes' ? initial : 'mixed')
  const [size, setSize] = useState<5 | 10 | 15>(10)
  const [busy, setBusy] = useState(false)
  const [actionError, setActionError] = useState<string | null>(null)
  const [confirmStartOver, setConfirmStartOver] = useState(false)

  if (query.loading) return <LoadingState label="Building your suggested review locally…" />
  if (query.error || !query.data) return <ErrorState message={query.error ?? 'Review data unavailable.'} onRetry={query.reload} />
  const data = query.data
  const available = mode === 'vocabulary' ? data.vocabulary.totalEligibleCount : mode === 'mistakes' ? data.recurringMistakes.confirmedCount : data.vocabulary.totalEligibleCount + data.recurringMistakes.confirmedCount

  const start = async (startOver = false) => {
    setBusy(true); setActionError(null)
    try { const session = await startReviewSession({ mode, itemCount: size, startOver }); navigate(`/review/session/${session.id}`) }
    catch (error) { setActionError(error instanceof Error ? error.message : String(error)) }
    finally { setBusy(false) }
  }

  return <PageShell width="wide">
    <PageHero eyebrow="Good to see you" title="Review what matters" accent="right now." description="A deterministic queue built from your real vocabulary and confirmed recurring mistakes." compact />

    {data.activeSession && <AppCard className="active-review-card mb-4">
      <div><p className="eyebrow">Review in progress</p><h2 className="section-title">{data.activeSession.reviewedItemCount} of {data.activeSession.actualItemCount} items reviewed</h2></div>
      <div className="flex flex-wrap gap-3"><button onClick={() => navigate(`/review/session/${data.activeSession!.id}`)} className="button-primary">Resume Review</button><button onClick={() => setConfirmStartOver(true)} className="button-secondary">Start Over</button></div>
    </AppCard>}

    <div className="review-layout">
      <AppCard>
        <SectionHeader title="Recommended Review Queue" description="The local review engine selects the most valuable eligible items for you." />
        <fieldset className="mt-6"><legend className="form-label mb-3">Review mode</legend><div className="review-mode-grid">{REVIEW_MODES.map((item) => {
          const Icon = item.icon
          return <label key={item.value} className={`review-mode-card ${mode === item.value ? 'is-selected' : ''}`}>
            <input aria-label={item.label} type="radio" name="mode" value={item.value} checked={mode === item.value} onChange={() => setMode(item.value)} />
            <Icon aria-hidden="true" size={30} /><strong>{item.label}</strong><span>{item.description}</span>
          </label>
        })}</div></fieldset>

        <fieldset className="mt-7"><legend className="form-label mb-3">Session size</legend><div className="session-size-row">{([5, 10, 15] as const).map((value) => <label key={value} className={size === value ? 'is-selected' : ''}><input type="radio" name="size" checked={size === value} onChange={() => setSize(value)} />{value} items</label>)}</div></fieldset>

        <div className="eligible-summary mt-5"><Info aria-hidden="true" size={18} /><div><strong>{available} eligible item{available === 1 ? '' : 's'}</strong><p>If fewer than {size} are available, the session uses what’s available.</p></div></div>
        {available === 0 ? <p role="status" className="notice notice-warning mt-4">Nothing needs review yet for this mode.</p> : !data.activeSession && <button disabled={busy} onClick={() => void start()} className="button-primary mt-5"><Play size={17} fill="currentColor" /> Start Review</button>}
        {actionError && <p role="alert" className="notice notice-error mt-4">{actionError}</p>}
      </AppCard>

      <aside className="grid content-start gap-4">
        <AppCard>
          <SectionHeader title="Review Summary" />
          <dl className="review-summary-grid"><Count icon={<BookOpen />} label="Items to Review" value={data.vocabulary.totalEligibleCount + data.recurringMistakes.confirmedCount} /><Count icon={<AlertTriangle />} label="Recurring Mistakes" value={data.recurringMistakes.confirmedCount} /><Count icon={<BookOpen />} label="Vocabulary" value={data.vocabulary.totalEligibleCount} /><Count icon={<History />} label="Reviewed Sessions" value={data.reviewHistory.completedSessionCount} /></dl>
          {data.reviewHistory.lastReviewAt && <p className="review-last-date"><Clock3 size={15} /> Last review: {formatLocalDate(data.reviewHistory.lastReviewAt)}</p>}
        </AppCard>
        <AppCard>
          <div className="flex items-center gap-2 text-[var(--accent)]"><Target size={18} /><h2 className="section-title text-base">Suggested focus</h2></div>
          <p className="section-description mt-3">Based on your real review history and analyzed learning data.</p>
          <div className="suggested-focus"><Brain size={18} /><strong>{data.suggestedFocus ?? 'Complete an analyzed lesson to receive a suggested focus.'}</strong></div>
        </AppCard>
      </aside>
    </div>

    <AppCard className="mt-4"><SectionHeader title="Recent Review Sessions" description="Your local self-assessment history." />{data.recentSessions.length === 0 ? <p className="section-description mt-4">No review history yet.</p> : <div className="review-history-list mt-4">{data.recentSessions.map((item) => <div key={item.id}><span>{formatLocalDate(item.startedAt)} · {humanize(item.mode)}</span><strong>{item.reviewedItemCount}/{item.actualItemCount} · {humanize(item.status)}</strong></div>)}</div>}</AppCard>
    <p className="page-footnote"><RotateCcw size={13} /> Review records self-assessment history only; it is not a score or SRS schedule.</p>
    <ConfirmationDialog open={confirmStartOver} title="Start this Review over?" description="Reviewed items remain in history. A new queue will replace the current active session." confirmLabel="Start Over" danger busy={busy} onClose={() => setConfirmStartOver(false)} onConfirm={() => void start(true)} />
  </PageShell>
}

function Count({ icon, label, value }: { icon: React.ReactNode; label: string; value: number }) { return <div className="review-count"><span aria-hidden="true">{icon}</span><div><dt>{label}</dt><dd>{value}</dd></div></div> }
