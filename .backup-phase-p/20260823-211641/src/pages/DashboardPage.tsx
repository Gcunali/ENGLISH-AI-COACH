import { useCallback, useEffect, useMemo, useState } from 'react'
import { Activity, BookOpen, Clock3, Mic2, Play, ShieldCheck, Sparkles, Square, WifiOff } from 'lucide-react'
import { Link } from 'react-router-dom'
import { LessonAnalysisReport } from '../components/LessonAnalysisReport'
import { PlacementSummaryCard } from '../components/PlacementSummaryCard'
import { PracticeConsistency } from '../components/PracticeConsistency'
import { ReviewOverviewCard } from '../components/ReviewOverviewCard'
import { EmptyState, ErrorState, LoadingState } from '../components/PageState'
import { useDashboardData, useLearningMemorySummaryData } from '../hooks/useLearningData'
import { useLessonAnalysis } from '../hooks/useLessonAnalysis'
import { useLocalVoiceEngine } from '../hooks/useLocalVoiceEngine'
import { probeLocalVoiceEngine } from '../services/native'
import { correctionCountForDisplay, useConversationStore } from '../stores/conversation'
import type { LocalVoiceEngineProbe } from '../types'
import { formatDuration, formatLocalDate, humanize, lessonTitle } from '../utils/format'

const EMPTY_PROBE: LocalVoiceEngineProbe = {
  projectRoot: '', localAiRoot: '',
  whisper: { cliFound: false, cliPath: '', streamFound: false, streamPath: '', modelFound: false, modelPath: '', modelName: 'ggml-small.en-q5_1.bin', threads: 12, additionalModels: [] },
  ollama: { reachable: false, baseUrl: 'http://127.0.0.1:11434', modelFound: false, modelName: 'qwen3.5:4b' },
  piper: { pythonFound: false, pythonPath: '', installed: false, version: null, voiceFound: false, voiceConfigFound: false, voiceModelPath: '', voiceConfigPath: '', voiceName: 'en_US-lessac-medium' },
  voiceDefaults: { whisperModel: 'ggml-small.en-q5_1.bin', whisperThreads: 12, silenceToStopSeconds: 3.5, preRollSeconds: 0.4, startVoiceBlocks: 3, minimumVoiceThreshold: 350, noiseMultiplier: 3, piperVoice: 'en_US-lessac-medium', ttsStartSilenceSeconds: 0.5, ollamaModel: 'qwen3.5:4b', ollamaThinking: false },
  optionalComponents: { sileroFound: false, sileroPath: '' }, offlineReady: false, problems: [],
}

const STATE_LABELS = {
  IDLE: 'Ready when you are', PREPARING: 'Calibrating microphone…', LISTENING: 'Listening…',
  STUDENT_SPEAKING: 'You are speaking…', TRANSCRIBING: 'Transcribing locally…',
  TEACHER_THINKING: 'Teacher is thinking…', TEACHER_SPEAKING: 'Teacher is speaking…',
  TEACHER_CANCELLING: 'Stopping teacher response…',
  PAUSED: 'Lesson paused', ENDING: 'Stopping voice engine…', ANALYZING: 'Analyzing locally…',
  COMPLETED: 'Lesson complete', ERROR: 'Local component unavailable',
} as const

const SCORE_LABELS = [
  ['fluency', 'Fluency'], ['grammar', 'Grammar'], ['vocabulary', 'Vocabulary'],
  ['comprehension', 'Comprehension'], ['interaction', 'Interaction'],
] as const

export function DashboardPage() {
  const dashboard = useDashboardData()
  const memory = useLearningMemorySummaryData()
  const [probe, setProbe] = useState(EMPTY_PROBE)
  const [showTranscript, setShowTranscript] = useState(true)
  const state = useConversationStore((store) => store.state)
  const messages = useConversationStore((store) => store.messages)
  const lesson = useConversationStore((store) => store.lesson)
  const correctionCandidates = useConversationStore((store) => store.correctionCandidates)
  const summary = useConversationStore((store) => store.summary)
  const metrics = useConversationStore((store) => store.metrics)
  const voiceTurnMetrics = useConversationStore((store) => store.voiceTurnMetrics)
  const streamedTeacherText = useConversationStore((store) => store.streamedTeacherText)
  const activeGenerationId = useConversationStore((store) => store.activeGenerationId)
  const error = useConversationStore((store) => store.error)
  const voice = useLocalVoiceEngine()
  const lessonAnalysis = useLessonAnalysis()
  const [clock, setClock] = useState(Date.now())

  const refreshProbe = useCallback(async () => setProbe(await probeLocalVoiceEngine()), [])
  useEffect(() => { void refreshProbe() }, [refreshProbe])
  const active = ['starting', 'running', 'stopping'].includes(voice.engineState)
  useEffect(() => {
    if (!active) return undefined
    const timer = window.setInterval(() => setClock(Date.now()), 1000)
    return () => window.clearInterval(timer)
  }, [active])
  const elapsed = summary?.durationSeconds ?? (lesson ? Math.max(0, Math.floor((clock - Date.parse(lesson.startedAt)) / 1000)) : 0)
  const correctedMessages = useMemo(() => new Set(correctionCandidates.map((item) => item.teacherMessageId)), [correctionCandidates])

  return <>
    <header className="flex flex-wrap items-center justify-between gap-4 mb-7">
      <div><p className="muted text-xs uppercase tracking-[.18em] mb-2">Home dashboard</p><h1 className="text-2xl md:text-3xl font-semibold tracking-tight m-0">Practice and review your real progress.</h1></div>
      <div className="flex items-center gap-2 rounded-full border border-white/10 px-3 py-2 text-xs"><WifiOff size={14} className="text-[var(--accent)]" /> SQLite + local AI only</div>
    </header>

    {dashboard.loading && <LoadingState label="Loading dashboard from this computer…" />}
    {dashboard.error && <ErrorState message={dashboard.error} onRetry={dashboard.reload} />}
    {dashboard.data && <section aria-label="Practice totals" className="mb-5 grid gap-3 sm:grid-cols-2 xl:grid-cols-5">
      <FactCard label="Total lessons" value={String(dashboard.data.totalLessons)} detail={`${dashboard.data.completedLessons} completed`} />
      <FactCard label="Practice time" value={dashboard.data.totalPracticeSeconds === null ? 'Not available' : formatDuration(dashboard.data.totalPracticeSeconds)} detail="Completed + interrupted" />
      <FactCard label="Your turns" value={String(dashboard.data.totalStudentTurns)} detail="Persisted student turns" />
      <FactCard label="Corrections" value={String(dashboard.data.totalCorrections)} detail="Live correction cues" />
      <FactCard label="Average score" value={dashboard.data.averageOverallScore === null ? 'Not available' : String(dashboard.data.averageOverallScore)} detail={`${dashboard.data.analyzedLessons} analyzed`} />
    </section>}
    {memory.data && <section aria-label="Learning memory summary" className="mb-5 grid gap-3 sm:grid-cols-2"><FactCard label="Vocabulary tracked" value={String(memory.data.vocabularyTotal)} detail={`${memory.data.vocabularyLearning} learning · ${memory.data.vocabularyKnown} known`} /><FactCard label="Recurring mistakes" value={String(memory.data.recurringMistakesConfirmed)} detail="Confirmed across 2+ lessons" /></section>}
    <div className="mb-5"><PlacementSummaryCard compact /></div>
    <div className="mb-5"><PracticeConsistency compact /></div>
    <div className="mb-5"><ReviewOverviewCard /></div>
    {dashboard.data?.totalLessons === 0 && <div className="mb-5"><EmptyState title="No lessons yet" message="Start your first local conversation below. Historical metrics will appear after real data is saved." /></div>}

    <div className="content-grid grid grid-cols-[minmax(0,1fr)_330px] gap-5">
      <section className="glass rounded-[28px] min-h-[560px] p-5 md:p-8 flex flex-col" aria-label="Conversation workspace">
        <div className="flex items-center justify-between gap-3 text-sm"><span className="rounded-full bg-white/5 px-3 py-1.5">Free conversation</span><span className="muted flex items-center gap-1.5"><Clock3 size={14} /> {formatDuration(elapsed)}</span></div>
        <div className="flex-1 flex flex-col items-center justify-center py-10">
          <div className="relative h-40 w-40 grid place-items-center">
            {active && <div className="pulse-ring absolute inset-2 rounded-full border border-[var(--accent)]/40" />}
            <div className="absolute inset-5 rounded-full bg-gradient-to-br from-[#1d2634] to-[#0c1017] border border-white/10" />
            <Mic2 className="relative text-[var(--accent)]" size={42} />
          </div>
          <p className="mt-5 mb-1 text-sm font-semibold tracking-[.12em] uppercase">{STATE_LABELS[state]}</p>
          <p className="muted text-sm text-center">{error ?? (probe.offlineReady ? 'Everything is processed on this device.' : 'Local voice components are not ready yet.')}</p>
          {!active ? <Link to="/lesson/new" aria-disabled={!probe.offlineReady || voice.engineState === 'starting'} className={`mt-5 w-fit bg-[var(--accent)] text-[#081006] font-semibold rounded-full px-6 py-3 flex items-center gap-2 no-underline ${!probe.offlineReady || voice.engineState === 'starting' ? 'pointer-events-none opacity-50' : ''}`}><Play size={17} fill="currentColor" /> Start Lesson</Link>
            : <div className="mt-5 flex flex-wrap justify-center gap-3">
              {activeGenerationId && ['TEACHER_THINKING', 'TEACHER_SPEAKING', 'TEACHER_CANCELLING'].includes(state) && <button type="button" aria-label="Stop teacher response" disabled={state === 'TEACHER_CANCELLING'} onClick={() => void voice.cancelTeacherResponse()} className="rounded-full border border-amber-300/25 px-5 py-3 text-amber-100 disabled:opacity-50"><Square size={14} className="mr-2 inline" fill="currentColor" />{state === 'TEACHER_CANCELLING' ? 'Stopping response…' : 'Stop response'}</button>}
              <button type="button" onClick={() => void voice.endLesson()} className="rounded-full border border-red-300/20 px-5 py-3 text-red-200"><Square size={14} className="mr-2 inline" fill="currentColor" /> End Lesson</button>
            </div>}
          {summary && <div className="mt-5 w-full max-w-xl rounded-2xl border border-white/[.08] bg-black/20 p-4" aria-label="Lesson summary">
            <div className="flex justify-between"><strong className="text-sm">Lesson saved</strong><span className="muted text-xs">{humanize(summary.status)}</span></div>
            <div className="mt-3 grid grid-cols-2 gap-2 sm:grid-cols-4"><MiniValue label="Duration" value={formatDuration(summary.durationSeconds)} /><MiniValue label="Your turns" value={String(summary.studentTurns)} /><MiniValue label="Teacher turns" value={String(summary.teacherTurns)} /><MiniValue label="Corrections" value={String(correctionCountForDisplay(summary, correctionCandidates))} /></div>
          </div>}
        </div>
        <div className="border-t border-white/[.07] pt-4">
          <button onClick={() => setShowTranscript((value) => !value)} className="w-full bg-transparent border-0 p-0 text-left text-sm text-white">Live transcript <span className="muted">{showTranscript ? 'On' : 'Off'}</span></button>
          {showTranscript && <div className="mt-4 max-h-52 overflow-auto space-y-3" aria-live="polite">
            {messages.length === 0 && !streamedTeacherText ? <p className="muted text-sm">Your conversation will appear here.</p> : messages.map((message) => <div key={message.id} className="grid grid-cols-[64px_1fr] gap-3 text-sm"><strong className={message.role === 'teacher' ? 'text-[var(--accent)] text-xs' : 'text-sky-300 text-xs'}>{message.role === 'teacher' ? 'TEACHER' : 'YOU'}</strong><span>{message.text}{correctedMessages.has(message.id) && <span className="ml-2 rounded-full bg-amber-300/10 px-2 py-0.5 text-[10px] text-amber-200">Correction</span>}</span></div>)}
            {streamedTeacherText && <div data-testid="teacher-draft" className="grid grid-cols-[64px_1fr] gap-3 text-sm opacity-75"><strong className="text-[var(--accent)] text-xs">TEACHER</strong><span>{streamedTeacherText}<span className="muted ml-2 text-[10px]">speaking…</span></span></div>}
          </div>}
        </div>
      </section>

      <aside className="space-y-5">
        <section className="glass rounded-[22px] p-5"><div className="flex items-center justify-between"><h2 className="m-0 text-base">Local system</h2><button onClick={() => void refreshProbe()} aria-label="Refresh diagnostics" className="rounded-lg border border-white/10 bg-white/5 p-2 text-white"><Activity size={16} /></button></div>
          <div className="mt-4 space-y-3"><Status label="Voice engine" ready={probe.offlineReady} /><Status label="Whisper" ready={probe.whisper.cliFound && probe.whisper.modelFound} /><Status label="Qwen" ready={probe.ollama.modelFound} /><Status label="Piper" ready={probe.piper.installed && probe.piper.voiceFound} /><Status label="Local database" ready /></div>
        </section>
        <section className="glass rounded-[22px] p-5"><h2 className="mt-0 text-base">Current session</h2><div className="grid grid-cols-2 gap-2"><MiniValue label="Your turns" value={String(messages.filter((item) => item.role === 'student').length)} /><MiniValue label="Corrections" value={String(correctionCountForDisplay(summary, correctionCandidates))} /><MiniValue label="STT" value={voiceTurnMetrics?.sttMs != null ? `${voiceTurnMetrics.sttMs} ms` : metrics ? `${metrics.sttMs} ms` : '—'} /><MiniValue label="First audio" value={voiceTurnMetrics?.captureEndToFirstAudioMs != null ? `${voiceTurnMetrics.captureEndToFirstAudioMs} ms` : '—'} /></div></section>
        <section className="glass rounded-[22px] p-5"><div className="flex items-center gap-2 text-sm"><ShieldCheck size={16} className="text-[var(--accent)]" /> No cloud storage</div><p className="muted mb-0 text-xs leading-5">Dashboard values come directly from the local SQLite database.</p></section>
      </aside>
    </div>

    {dashboard.data?.latestLesson && <section className="glass mt-5 rounded-[28px] p-5 md:p-7" aria-label="Latest lesson"><div className="flex flex-wrap items-center justify-between gap-4"><div><p className="muted mb-1 text-[10px] uppercase tracking-widest">Latest lesson · {dashboard.data.latestLesson.modeTitle}</p><h2 className="m-0 text-lg">{dashboard.data.latestLesson.customTitle || lessonTitle(dashboard.data.latestLesson.topic)}</h2><p className="muted mb-0 text-sm">{formatLocalDate(dashboard.data.latestLesson.startedAt)} · {formatDuration(dashboard.data.latestLesson.durationSeconds)} · {dashboard.data.latestLesson.studentTurnCount} student turns · {humanize(dashboard.data.latestLesson.status)}</p></div><div className="flex items-center gap-3">{dashboard.data.latestLesson.overallScore !== null && <strong className="text-2xl text-[var(--accent)]">{dashboard.data.latestLesson.overallScore}</strong>}<Link to={`/history/${dashboard.data.latestLesson.id}`} className="rounded-full border border-white/15 px-4 py-2 text-sm text-white no-underline">View lesson</Link></div></div></section>}

    {dashboard.data?.latestAnalyzedLesson && <section className="glass mt-5 rounded-[28px] p-5 md:p-7" aria-label="Current performance"><div className="flex items-center gap-2"><Sparkles size={16} className="text-[var(--accent)]" /><h2 className="m-0 text-lg">Current performance</h2><span className="muted text-xs">Latest analysis</span></div><div className="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-5">{SCORE_LABELS.map(([key, label]) => <FactCard key={key} label={label} value={String(dashboard.data!.latestAnalyzedLesson!.scores[key])} />)}</div></section>}
    {dashboard.data?.latestRecommendation && <section className="glass mt-5 rounded-[28px] p-5 md:p-7"><div className="flex items-start gap-3"><BookOpen className="mt-1 text-[var(--accent)]" size={18} /><div><p className="muted mb-1 text-[10px] uppercase tracking-widest">Suggested next focus · latest analysis</p><p className="m-0 text-sm">{dashboard.data.latestRecommendation}</p></div></div></section>}
    {summary && <LessonAnalysisReport analysis={lessonAnalysis.analysis} status={lessonAnalysis.status} error={lessonAnalysis.error} onRetry={() => void lessonAnalysis.retry()} />}
  </>
}

function FactCard({ label, value, detail }: { label: string; value: string; detail?: string }) {
  return <div className="glass rounded-2xl p-4"><div className="muted text-[10px] uppercase tracking-wider">{label}</div><div className="mt-2 text-2xl font-semibold">{value}</div>{detail && <div className="muted mt-1 text-[11px]">{detail}</div>}</div>
}
function MiniValue({ label, value }: { label: string; value: string }) { return <div className="rounded-xl bg-white/[.035] p-3 text-center"><div className="font-semibold text-sm">{value}</div><div className="muted mt-1 text-[10px]">{label}</div></div> }
function Status({ label, ready }: { label: string; ready: boolean }) { return <div className="flex items-center gap-3 text-sm"><span className={`status-dot ${ready ? 'bg-[var(--accent)]' : 'bg-amber-400'}`} /><span>{label}</span><span className="muted ml-auto text-xs">{ready ? 'Ready' : 'Unavailable'}</span></div> }
