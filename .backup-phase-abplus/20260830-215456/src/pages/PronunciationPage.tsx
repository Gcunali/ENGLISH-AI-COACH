import { AudioLines, CheckCircle2, ChevronDown, ChevronUp, Clock3, Mic, RotateCcw, ShieldCheck, Square, Volume2 } from 'lucide-react'
import { useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { AppCard, InlineNotice, PageHero, PageShell, SectionHeader, StatusBadge } from '../components/ProductUI'
import { usePronunciationPractice, validatePronunciationTarget } from '../hooks/usePronunciationPractice'
import type { PronunciationAttempt } from '../types'
import { formatLocalDate } from '../utils/format'

export function PronunciationPage() {
  const [params] = useSearchParams()
  const source = params.get('text') ?? ''
  const sourceId = params.get('sourceId')
  const sourceType = params.get('source') === 'vocabulary' && sourceId ? 'vocabulary' : 'custom'
  const practice = usePronunciationPractice(source, sourceType, sourceId)
  const validation = validatePronunciationTarget(practice.target)
  const unavailable = practice.state === 'error' && !practice.engine?.ready
  const busy = practice.state === 'recording' || practice.state === 'checking' || practice.state === 'analyzing'

  return <PageShell width="wide">
    <PageHero eyebrow="Let’s make your pronunciation clear and confident" title="Pronunciation" accent="Practice" description="Practice a word or short phrase and get local acoustic feedback without changing your lesson score." compact illustration="background" />

    <AppCard>
      <div className="pronunciation-input-layout">
        <div>
          <p className="eyebrow">Practice input</p>
          <label className="form-label mt-4" htmlFor="pronunciation-target">Target word or phrase</label>
          <textarea id="pronunciation-target" aria-describedby="pronunciation-help" aria-invalid={!!practice.target && !!validation} value={practice.target} onChange={(event) => practice.setTarget(event.target.value)} maxLength={160} rows={3} disabled={busy} placeholder="I'm terrible at cooking." className="form-control mt-2" />
          <div id="pronunciation-help" className="pronunciation-help"><span className={practice.target && validation ? 'text-red-200' : undefined}>{practice.target && validation ? validation : 'General American English reference · maximum 12 words'}</span><span>{practice.target.length}/160</span></div>
          <PracticeStatus state={practice.state} />
          {practice.error && <p role="alert" className="notice notice-error mt-4">{practice.error}</p>}
          {unavailable && <div className="mt-4"><InlineNotice tone="warning" title="Pronunciation is unavailable">Check System Diagnostics for the local acoustic model status. Other learning features remain available.</InlineNotice></div>}
          <div className="pronunciation-actions">
            {practice.state === 'recording' ? <button className="button-primary" onClick={() => void practice.stop()}><Square size={16} fill="currentColor" /> Stop recording</button> : <button className="button-primary" aria-busy={practice.state === 'checking' || practice.state === 'analyzing'} disabled={!!validation || !practice.engine?.ready || practice.state === 'checking' || practice.state === 'analyzing'} onClick={() => void practice.start()}><Mic size={17} /> Record pronunciation</button>}
            {(practice.state === 'checking' || practice.state === 'analyzing') && <button className="button-secondary" onClick={() => void practice.cancel()}>Cancel</button>}
            <button className="button-secondary" disabled aria-describedby="hear-target-help"><Volume2 size={16} /> Hear target</button>
          </div>
          <p id="hear-target-help" className="page-footnote">Target playback remains unavailable while acoustic scoring calibration is protected.</p>
        </div>
        <aside className="session-status-panel">
          <p className="eyebrow">Session status</p>
          <SessionLine icon={<Mic />} label="Microphone" value={practice.engine?.ready ? 'Ready' : 'Unavailable'} tone={practice.engine?.ready ? 'success' : 'warning'} />
          <SessionLine icon={<AudioLines />} label="Analysis" value={busy ? 'Active' : 'Ready on demand'} tone={busy ? 'info' : 'neutral'} />
          <SessionLine icon={<ShieldCheck />} label="Feedback mode" value="Local acoustic" tone="info" />
        </aside>
      </div>
    </AppCard>

    <div className="pronunciation-results-layout">
      <History items={practice.history} />
      {practice.result ? <ResultCard attempt={practice.result} retry={practice.retry} /> : <AppCard><SectionHeader title="Pronunciation Score" /><div className="empty-score"><AudioLines size={28} /><strong>Your result will appear here</strong><span>Record the target phrase to receive local acoustic feedback.</span></div></AppCard>}
    </div>
    <p className="page-footnote"><ShieldCheck size={14} /> This is a local pronunciation-practice estimate, not a language certification. Accent variation, microphone quality and background noise may affect the score.</p>
  </PageShell>
}

function PracticeStatus({ state }: { state: string }) {
  const text = state === 'loading_engine' ? 'Loading local pronunciation model…' : state === 'recording' ? 'Recording… Speak the target, then stop.' : state === 'checking' ? 'Checking phrase…' : state === 'analyzing' ? 'Analyzing pronunciation…' : null
  return text ? <p role="status" className="practice-live-status"><AudioLines size={17} />{text}</p> : null
}

function SessionLine({ icon, label, value, tone }: { icon: React.ReactNode; label: string; value: string; tone: 'success' | 'warning' | 'info' | 'neutral' }) { return <div className="session-status-line"><span aria-hidden="true">{icon}</span><div><strong>{label}</strong><StatusBadge tone={tone}>{value}</StatusBadge></div></div> }

function ResultCard({ attempt, retry }: { attempt: PronunciationAttempt; retry: () => void }) {
  const [open, setOpen] = useState<number | null>(null)
  if (attempt.status === 'content_mismatch') return <AppCard><SectionHeader title="Try the phrase again" /><p className="section-description mt-3">The recording did not match the target closely enough. Try the phrase again.</p><button className="button-secondary mt-5" onClick={retry}><RotateCcw size={16} /> Try Again</button></AppCard>
  if (attempt.status !== 'completed') return <AppCard><SectionHeader title="No score available" /><p className="section-description mt-3">There was not enough reliable acoustic evidence. Please record again.</p><button className="button-secondary mt-5" onClick={retry}><RotateCcw size={16} /> Try Again</button></AppCard>
  const score = Math.round(attempt.overallScore ?? 0)
  return <AppCard>
    <SectionHeader title="Pronunciation Score" description={`Confidence: ${attempt.confidence}`} />
    <div className="score-circle" style={{ '--score': `${score * 3.6}deg` } as React.CSSProperties}><div><strong>{score}%</strong><span>{score >= 85 ? 'Great work!' : score >= 70 ? 'Good job!' : 'Keep practicing'}</span></div></div>
    <p className="target-summary">Target: <strong>{attempt.targetText}</strong></p>
    {attempt.confidence === 'low' && <InlineNotice tone="warning">Low-confidence acoustic evidence. Treat this result as guidance and try again in a quieter setting.</InlineNotice>}
    <div className="word-feedback-list">{attempt.words.map((word) => <div key={word.index}>
      <button aria-expanded={open === word.index} onClick={() => setOpen(open === word.index ? null : word.index)}><span><CheckCircle2 size={17} />{word.word}</span><span><strong>{Math.round(word.score)}</strong>{open === word.index ? <ChevronUp size={16} /> : <ChevronDown size={16} />}</span></button>
      {open === word.index && <div className="phone-feedback"><p>Expected sounds: {word.expectedPhones.join(' · ')}</p>{word.phoneResults.map((phone, index) => <div key={`${phone.phone}-${index}`}><div><code>/{phone.phone}/</code><strong>{Math.round(phone.score)}</strong></div>{phone.closestAlternative && <p>This part was acoustically closer to /{phone.closestAlternative}/ than the target /{phone.phone}/.</p>}{phone.hint && <p>{phone.hint}</p>}</div>)}</div>}
    </div>)}</div>
    <button className="button-secondary mt-5" onClick={retry}><RotateCcw size={16} /> Try Again</button>
  </AppCard>
}

function History({ items }: { items: PronunciationAttempt[] }) { return <AppCard className="recent-practice-card"><SectionHeader id="pronunciation-history" title="Recent Practice" />{!items.length ? <div className="empty-practice"><Clock3 size={24} /><strong>No pronunciation attempts yet.</strong><span>Record a short target and your local results will appear here.</span></div> : <div className="recent-attempts">{items.map((item) => <div key={item.id}><span className="attempt-play"><PlayIcon /></span><div className="min-w-0"><strong>{item.targetText}</strong><span>{formatLocalDate(item.createdAt)}</span></div><b>{item.overallScore === null ? 'No score' : `${Math.round(item.overallScore)}%`}</b></div>)}</div>}</AppCard> }
function PlayIcon() { return <span aria-hidden="true">▶</span> }
