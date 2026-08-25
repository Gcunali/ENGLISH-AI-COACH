import { useEffect, useState } from 'react'
import { Check, Circle, Mic, RotateCcw, Square } from 'lucide-react'
import { Link, useNavigate } from 'react-router-dom'
import { ErrorState, LoadingState } from '../components/PageState'
import { usePlacementOverview } from '../hooks/usePlacementOverview'
import { usePlacementRecorder } from '../hooks/usePlacementRecorder'
import { abandonPlacementTest, confirmPlacementSpeakingResponse, finalizePlacementTest, resumePlacementTest, skipPlacementSpeaking, startPlacementTest, submitPlacementAnswer } from '../services/native'
import type { PlacementSession } from '../types'
import { humanize } from '../utils/format'

export function PlacementPage() {
  const overview = usePlacementOverview()
  const [session, setSession] = useState<PlacementSession | null>(null)
  const [selected, setSelected] = useState('')
  const [busy, setBusy] = useState(false)
  const [actionError, setActionError] = useState<string | null>(null)
  const [confirmSkip, setConfirmSkip] = useState(false)
  const recorder = usePlacementRecorder()
  const navigate = useNavigate()
  useEffect(() => setSelected(''), [session?.question?.questionId])

  const action = async (work: () => Promise<PlacementSession>) => {
    setBusy(true); setActionError(null)
    try { setSession(await work()) } catch (value) { setActionError(value instanceof Error ? value.message : String(value)) }
    finally { setBusy(false) }
  }
  const finalize = async () => {
    if (!session) return
    setBusy(true); setActionError(null)
    try { const result = await finalizePlacementTest(session.attempt.id); navigate(`/placement/results/${result.attempt.id}`) }
    catch (value) { setActionError(value instanceof Error ? value.message : String(value)) }
    finally { setBusy(false) }
  }

  if (overview.loading) return <LoadingState label="Loading local Placement Test…" />
  if (overview.error) return <ErrorState message={overview.error} onRetry={overview.reload} />
  if (!session) return <Landing overview={overview.data!} busy={busy} error={actionError} onStart={() => void action(() => startPlacementTest(false))} onResume={(id) => void action(() => resumePlacementTest(id))} onStartOver={(id) => void action(async () => { await abandonPlacementTest(id); return startPlacementTest(false) })} />

  const enoughSpeaking = session.progress.speakingResponses >= 2 && session.progress.speakingWordCount >= 40
  return <>
    <header className="mb-6 flex flex-wrap items-start justify-between gap-4"><div><p className="muted text-xs uppercase tracking-[.18em] mb-2">Adaptive · local · version 1</p><h1 className="m-0 text-2xl md:text-3xl">Placement Test</h1><p className="muted text-sm">Answers are saved immediately. Submitted answers cannot be changed.</p></div><Link to="/" className="rounded-full border border-white/15 px-4 py-2 text-sm text-white no-underline">Exit and resume later</Link></header>
    <Progress session={session} />
    {actionError && <div role="alert" className="mb-4 rounded-xl border border-red-300/20 bg-red-300/[.06] p-4 text-sm text-red-100">{actionError}</div>}
    {session.question && <section className="glass rounded-[28px] p-5 md:p-8" aria-label="Placement question">
      <p className="muted text-xs uppercase tracking-widest">{humanize(session.question.skill)}</p>
      {session.question.passage && <article className="mb-6 rounded-2xl border border-sky-300/10 bg-sky-300/[.04] p-5 leading-7" aria-label="Reading passage">{session.question.passage}</article>}
      <h2 className="text-xl leading-8">{session.question.prompt}</h2>
      <fieldset className="mt-5 space-y-3"><legend className="sr-only">Choose one answer</legend>{session.question.options.map((option) => <label key={option.id} className={`flex cursor-pointer gap-3 rounded-2xl border p-4 ${selected === option.id ? 'border-[var(--accent)] bg-[var(--accent)]/[.07]' : 'border-white/10 bg-white/[.025]'}`}><input type="radio" name="placement-answer" value={option.id} checked={selected === option.id} onChange={() => setSelected(option.id)} /><span>{option.text}</span></label>)}</fieldset>
      <button disabled={!selected || busy} onClick={() => void action(() => submitPlacementAnswer(session.attempt.id, session.question!.questionId, selected))} className="mt-6 rounded-full bg-[var(--accent)] px-6 py-3 font-semibold text-black disabled:opacity-40">Submit answer</button>
    </section>}
    {!session.question && session.progress.phase === 'speaking' && <section className="glass rounded-[28px] p-5 md:p-8" aria-label="Spoken production">
      <p className="muted text-xs uppercase tracking-widest">Spoken Production · {session.progress.speakingResponses} of 3 confirmed</p>
      {session.speakingPrompt ? <><h2 className="max-w-3xl text-xl leading-8">{session.speakingPrompt.prompt}</h2><p className="muted text-sm">Whisper creates a local transcript. Pronunciation is not evaluated.</p>
        {recorder.state === 'listening' ? <button onClick={() => void recorder.stop()} className="rounded-full border border-red-300/30 px-5 py-3 text-red-100"><Square size={15} className="mr-2 inline" />Stop recording</button> : <button disabled={recorder.state === 'transcribing' || busy} onClick={() => void recorder.start()} className="rounded-full bg-[var(--accent)] px-5 py-3 font-semibold text-black disabled:opacity-40"><Mic size={16} className="mr-2 inline" />{recorder.state === 'transcribing' ? 'Transcribing locally…' : 'Record answer'}</button>}
        {recorder.error && <p role="alert" className="text-sm text-red-200">{recorder.error}</p>}
        {recorder.transcript && <div className="mt-5 rounded-2xl bg-white/[.04] p-5" aria-label="Transcript preview"><p className="muted mt-0 text-xs uppercase tracking-widest">Transcript preview</p><p>{recorder.transcript}</p><div className="flex flex-wrap gap-3"><button onClick={recorder.retry} className="rounded-full border border-white/15 px-4 py-2"><RotateCcw size={14} className="mr-2 inline" />Retry</button><button disabled={busy} onClick={() => void action(async () => { const next = await confirmPlacementSpeakingResponse(session.attempt.id, session.speakingPrompt!.promptId, recorder.transcript); recorder.retry(); return next })} className="rounded-full bg-[var(--accent)] px-4 py-2 font-semibold text-black"><Check size={14} className="mr-2 inline" />Confirm answer</button></div></div>}
      </> : <p>All three speaking prompts are confirmed.</p>}
      <div className="mt-6 flex flex-wrap gap-3">{enoughSpeaking && <button disabled={busy} onClick={() => void finalize()} className="rounded-full bg-[var(--accent)] px-5 py-3 font-semibold text-black">Evaluate speaking and finish</button>}{!confirmSkip ? <button onClick={() => setConfirmSkip(true)} className="rounded-full border border-white/15 px-5 py-3">Skip speaking section</button> : <div role="alert" className="rounded-2xl border border-amber-300/20 p-4 text-sm"><p className="mt-0">Speaking will be unavailable and overall confidence will be reduced.</p><button disabled={busy} onClick={() => void action(() => skipPlacementSpeaking(session.attempt.id))} className="rounded-full border border-amber-200/30 px-4 py-2">Confirm skip</button></div>}</div>
    </section>}
    {session.progress.phase === 'ready_to_finalize' && <section className="glass rounded-[28px] p-7 text-center"><h2>Objective domains complete</h2><p className="muted">Speaking is marked {humanize(session.attempt.speakingStatus)}. Final scoring is deterministic.</p><button disabled={busy} onClick={() => void finalize()} className="rounded-full bg-[var(--accent)] px-6 py-3 font-semibold text-black">Calculate estimated level</button></section>}
  </>
}

function Landing({ overview, busy, error, onStart, onResume, onStartOver }: { overview: NonNullable<ReturnType<typeof usePlacementOverview>['data']>; busy: boolean; error: string | null; onStart: () => void; onResume: (id: string) => void; onStartOver: (id: string) => void }) {
  return <><header className="mb-7"><p className="muted text-xs uppercase tracking-[.18em] mb-2">CEFR foundation</p><h1 className="m-0 text-2xl md:text-3xl">Placement Test</h1><p className="muted max-w-3xl">A local, adaptive estimate of Grammar, Vocabulary, Reading, and Spoken Production. Usually around 15–30 minutes depending on your level.</p></header>
    <section className="grid gap-4 md:grid-cols-2 xl:grid-cols-4" aria-label="Skills assessed">{['Grammar','Vocabulary','Reading','Spoken Production'].map((label) => <div key={label} className="glass rounded-2xl p-5"><Check size={18} className="text-[var(--accent)]" /><h2 className="text-base">{label}</h2><p className="muted mb-0 text-xs">Assessed in Placement v1</p></div>)}</section>
    <section className="glass mt-5 rounded-[28px] p-6"><h2 className="text-lg">Important limits</h2><p className="muted text-sm">Listening, pronunciation, and formal writing are not assessed. This is an internal CEFR-informed estimate, not an official certification.</p>{error && <p role="alert" className="text-red-200">{error}</p>}{overview.activeAttempt ? <div className="flex flex-wrap gap-3"><button disabled={busy} onClick={() => onResume(overview.activeAttempt!.id)} className="rounded-full bg-[var(--accent)] px-5 py-3 font-semibold text-black">Resume placement</button><button disabled={busy} onClick={() => onStartOver(overview.activeAttempt!.id)} className="rounded-full border border-white/15 px-5 py-3">Start over</button></div> : <button disabled={busy} onClick={onStart} className="rounded-full bg-[var(--accent)] px-5 py-3 font-semibold text-black">Start Placement Test</button>}</section>
    {overview.currentResult && <section className="glass mt-5 rounded-2xl p-5"><p className="muted text-xs uppercase tracking-widest">Current estimate</p><div className="text-3xl font-semibold text-[var(--accent)]">{overview.currentResult.estimatedCefrLevel}</div><p className="muted text-sm">Confidence: {humanize(overview.currentResult.confidence)}</p><Link to={`/placement/results/${overview.currentResult.attempt.id}`} className="text-white">View result</Link></section>}
  </>
}

function Progress({ session }: { session: PlacementSession }) { return <section className="mb-5 grid gap-3 sm:grid-cols-2 xl:grid-cols-4" aria-label="Placement progress">{session.progress.domains.map((domain) => <div key={domain.skill} className="glass rounded-2xl p-4"><div className="flex items-center gap-2"><Circle size={12} className={domain.status === 'complete' ? 'fill-[var(--accent)] text-[var(--accent)]' : 'text-white/40'} /><strong className="text-sm">{humanize(domain.skill)}</strong></div><p className="muted mb-0 text-xs">{humanize(domain.status)} · {domain.answeredQuestions} answered</p></div>)}<div className="glass rounded-2xl p-4"><div className="flex items-center gap-2"><Circle size={12} className={session.progress.phase !== 'objective' ? 'text-[var(--accent)]' : 'text-white/40'} /><strong className="text-sm">Speaking</strong></div><p className="muted mb-0 text-xs">{session.progress.speakingResponses} responses · {session.progress.speakingWordCount} words</p></div></section> }
