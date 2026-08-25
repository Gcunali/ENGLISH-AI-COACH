import { ArrowLeft, Clock3 } from 'lucide-react'
import { useMemo, useState } from 'react'
import { Link, useParams } from 'react-router-dom'
import { LessonAnalysisReport } from '../components/LessonAnalysisReport'
import { EmptyState, ErrorState, LoadingState } from '../components/PageState'
import { useLessonDetailsData } from '../hooks/useLearningData'
import { retryLessonAnalysis } from '../services/native'
import { formatDuration, formatLocalDate, humanize, lessonTitle } from '../utils/format'
import { notifyLearningDataChanged } from '../utils/learningData'

export function LessonDetailsPage() {
  const { lessonId = '' } = useParams()
  const query = useLessonDetailsData(lessonId)
  const [retryError, setRetryError] = useState<string | null>(null)
  const [retrying, setRetrying] = useState(false)
  const correctedTeacherMessages = useMemo(() => new Set(query.data?.correctionCandidates.map((item) => item.teacherMessageId) ?? []), [query.data])

  const retry = async () => {
    if (!query.data?.analysis || retrying) return
    setRetrying(true)
    setRetryError(null)
    try {
      await retryLessonAnalysis(query.data.lesson.id)
      query.reload()
      notifyLearningDataChanged()
    } catch (reason) {
      setRetryError(reason instanceof Error ? reason.message : String(reason))
      query.reload()
    } finally {
      setRetrying(false)
    }
  }

  if (query.loading) return <LoadingState label="Loading persisted lesson details…" />
  if (query.error) return <ErrorState message={query.error} onRetry={query.reload} />
  if (!query.data) return <EmptyState title="Lesson not found" message="This lesson does not exist in the local database." />
  const { lesson, messages, analysis, configuration } = query.data

  return <>
    <Link to="/history" className="muted mb-5 inline-flex items-center gap-2 text-sm no-underline"><ArrowLeft size={15} /> Back to History</Link>
    <header className="glass rounded-[28px] p-5 md:p-7"><div className="flex flex-wrap items-start justify-between gap-4"><div><p className="muted mb-1 text-[10px] uppercase tracking-widest">Lesson details</p><h1 className="m-0 text-2xl">{lessonTitle(lesson.topic)}</h1><p className="muted mb-0 mt-2 text-sm">{formatLocalDate(lesson.startedAt)}</p></div>{analysis?.overallScore !== null && analysis?.overallScore !== undefined && <div className="rounded-2xl bg-[var(--accent)]/10 px-5 py-3 text-center"><strong className="text-3xl text-[var(--accent)]">{analysis.overallScore}</strong><div className="muted text-[10px]">Overall / 100</div></div>}</div>
      <div className="mt-5 grid gap-3 sm:grid-cols-3 lg:grid-cols-6"><Detail label="Status" value={humanize(lesson.status)} /><Detail label="Duration" value={formatDuration(lesson.durationSeconds)} /><Detail label="Student turns" value={String(lesson.studentTurnCount)} /><Detail label="Teacher turns" value={String(lesson.teacherTurnCount)} /><Detail label="Corrections" value={String(lesson.correctionCount)} /><Detail label="Mode" value={configuration.modeTitle} /></div>
      <details className="mt-5 rounded-xl border border-white/[.07] bg-black/10 p-4"><summary className="cursor-pointer text-sm">Technical details</summary><dl className="mt-4 grid gap-3 text-xs sm:grid-cols-2 lg:grid-cols-4"><Technical label="Whisper" value={lesson.whisperModel} /><Technical label="Whisper threads" value={String(lesson.whisperThreads)} /><Technical label="Ollama" value={lesson.ollamaModel} /><Technical label="Piper" value={lesson.piperVoice} /></dl></details>
    </header>

    <section className="glass mt-5 rounded-[28px] p-5 md:p-7" aria-label="Lesson setup"><h2 className="mt-0 text-lg">Lesson Setup</h2><div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3"><Detail label="Mode" value={`${configuration.modeTitle}${configuration.legacy ? ' · Legacy' : ''}`} /><Detail label="Difficulty" value={humanize(configuration.difficulty)} />{configuration.topic && <Detail label="Topic" value={configuration.topic} />}{configuration.objective && <Detail label="Objective" value={configuration.objective} />}{configuration.scenario && <Detail label="Scenario" value={configuration.scenario} />}{configuration.focusAreas.length > 0 && <Detail label="Focus areas" value={configuration.focusAreas.map(humanize).join(', ')} />}</div></section>

    <section className="glass mt-5 rounded-[28px] p-5 md:p-7" aria-labelledby="transcript-heading"><div className="flex items-center gap-2"><Clock3 size={16} className="text-[var(--accent)]" /><h2 id="transcript-heading" className="m-0 text-lg">Full transcript</h2></div>
      {messages.length === 0 ? <p className="muted mb-0 mt-4 text-sm">No transcript messages were persisted for this lesson.</p> : <div className="mt-5 space-y-4">{messages.map((message) => <article key={message.id} className="grid gap-2 border-b border-white/[.06] pb-4 sm:grid-cols-[90px_1fr]"><div><strong className={message.role === 'teacher' ? 'text-[var(--accent)] text-xs' : 'text-sky-300 text-xs'}>{message.role === 'teacher' ? 'TEACHER' : 'YOU'}</strong>{correctedTeacherMessages.has(message.id) && <div className="mt-1 inline-block rounded-full bg-amber-300/10 px-2 py-0.5 text-[10px] text-amber-200">Correction</div>}</div><p className="m-0 whitespace-pre-wrap text-sm leading-6">{message.text}</p></article>)}</div>}
    </section>

    {!analysis && lesson.status === 'completed' && <EmptyState title="Not analyzed" message="No persisted analysis exists for this lesson." />}
    {!analysis && lesson.status !== 'completed' && <EmptyState title={`Lesson ${humanize(lesson.status).toLowerCase()}`} message="Analysis is unavailable for interrupted or failed lessons. The partial transcript remains preserved." />}
    {analysis && <LessonAnalysisReport analysis={analysis} status={retrying ? 'running' : analysis.status} error={retryError} onRetry={() => void retry()} />}
  </>
}

function Detail({ label, value }: { label: string; value: string }) { return <div className="rounded-xl bg-white/[.035] p-3"><div className="font-semibold text-sm">{value}</div><div className="muted mt-1 text-[10px]">{label}</div></div> }
function Technical({ label, value }: { label: string; value: string }) { return <div><dt className="muted">{label}</dt><dd className="m-0 mt-1 break-all">{value}</dd></div> }
