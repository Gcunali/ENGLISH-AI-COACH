import type { LessonAnalysis, LessonAnalysisStatus } from '../types'

interface Props {
  analysis: LessonAnalysis | null
  status: LessonAnalysisStatus | null
  error: string | null
  onRetry: () => void
}

const SCORE_LABELS = [
  ['fluency', 'Fluency'],
  ['grammar', 'Grammar'],
  ['vocabulary', 'Vocabulary'],
  ['comprehension', 'Comprehension'],
  ['interaction', 'Interaction'],
] as const

export function LessonAnalysisReport({ analysis, status, error, onRetry }: Props) {
  if (!status) return null

  if (status === 'pending' || status === 'running') {
    return <AnalysisShell><div role="status" className="flex items-center gap-3 text-sm"><span className="h-4 w-4 animate-spin rounded-full border-2 border-white/20 border-t-[var(--accent)]" />Analyzing your lesson locally…</div></AnalysisShell>
  }

  if (status === 'insufficient_data') {
    return <AnalysisShell><h2 className="mt-0 text-lg">Lesson Analysis</h2><p className="muted mb-0 text-sm">Not enough data for reliable scores. Complete at least three valid student turns.</p></AnalysisShell>
  }

  if (status === 'failed') {
    return <AnalysisShell><h2 className="mt-0 text-lg">Lesson Analysis</h2><p className="text-sm text-red-200">Analysis failed. Your lesson and transcript remain saved.</p><p className="muted text-xs">{analysis?.errorMessage ?? error ?? 'Local analyzer unavailable.'}</p><button onClick={onRetry} className="rounded-full border border-white/15 bg-white/5 px-4 py-2 text-sm text-white">Retry analysis</button></AnalysisShell>
  }

  if (!analysis || !analysis.scores || analysis.overallScore === null) {
    return <AnalysisShell><p className="text-sm text-red-200">The completed analysis could not be displayed.</p></AnalysisShell>
  }

  return <AnalysisShell>
    <div className="flex flex-wrap items-start justify-between gap-4">
      <div><p className="muted mb-1 text-[10px] uppercase tracking-[.18em]">Local pedagogical report</p><h2 className="m-0 text-xl">Lesson Analysis</h2></div>
      <div className="rounded-2xl bg-[var(--accent)]/10 px-5 py-3 text-center"><div className="text-3xl font-semibold text-[var(--accent)]">{analysis.overallScore}</div><div className="muted text-[10px]">Overall / 100</div></div>
    </div>

    <p className="mt-5 text-sm leading-6">{analysis.summary}</p>
    <p className="muted text-[11px]">Internal pedagogical estimate from this lesson—not an official proficiency certification.</p>

    <div className="mt-5 grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-5">
      {SCORE_LABELS.map(([key, label]) => <ScoreCard key={key} label={label} value={analysis.scores![key]} />)}
    </div>
    <div className="mt-3 rounded-xl bg-white/[.035] p-3 text-sm"><span className="font-medium">Pronunciation</span><span className="muted ml-2">Not evaluated yet — acoustic analysis is unavailable.</span></div>

    {analysis.strengths.length > 0 && <ReportSection title="Strengths">{analysis.strengths.map((item) => <ReportItem key={`${item.title}-${item.evidence}`} title={item.title}><p>{item.evidence}</p></ReportItem>)}</ReportSection>}

    {analysis.priorityImprovements.length > 0 && <ReportSection title="Priority improvements">{analysis.priorityImprovements.map((item) => <ReportItem key={`${item.area}-${item.title}`} title={item.title} tag={item.area}><p>{item.explanation}</p><Example label="From this lesson" value={item.exampleFromLesson} /><Example label="Better alternative" value={item.betterAlternative} accent /></ReportItem>)}</ReportSection>}

    {analysis.corrections.length > 0 && <ReportSection title="Corrections">{analysis.corrections.map((item) => <ReportItem key={`${item.original}-${item.corrected}`} title={item.category.replaceAll('_', ' ')}><Example label="You said" value={item.original} /><Example label="Better" value={item.corrected} accent /><p>{item.explanation}</p></ReportItem>)}</ReportSection>}

    {analysis.naturalAlternatives.length > 0 && <ReportSection title="Natural alternatives">{analysis.naturalAlternatives.map((item) => <ReportItem key={`${item.original}-${item.alternative}`} title={item.original}><Example label="Natural option" value={item.alternative} accent /></ReportItem>)}</ReportSection>}

    {analysis.vocabulary.length > 0 && <ReportSection title="Vocabulary from this lesson">{analysis.vocabulary.map((item) => <ReportItem key={item.wordOrPhrase} title={item.wordOrPhrase}><p>{item.meaning}</p><Example label="Example" value={item.example} /></ReportItem>)}</ReportSection>}

    {analysis.recurringPatterns.length > 0 && <ReportSection title="Recurring patterns">{analysis.recurringPatterns.map((item) => <ReportItem key={item.pattern} title={item.pattern} tag={`${item.count} occurrences`}><p>{item.explanation}</p></ReportItem>)}</ReportSection>}

    {analysis.nextLessonRecommendations.length > 0 && <ReportSection title="Recommended focus for next lesson"><ul className="mb-0 space-y-2 pl-5 text-sm">{analysis.nextLessonRecommendations.map((item) => <li key={item}>{item}</li>)}</ul></ReportSection>}
  </AnalysisShell>
}

function AnalysisShell({ children }: { children: React.ReactNode }) {
  return <section className="glass mt-5 rounded-[28px] p-5 md:p-8" aria-label="Lesson analysis">{children}</section>
}

function ScoreCard({ label, value }: { label: string; value: number }) {
  return <div className="rounded-xl bg-white/[.035] p-3"><div className="flex items-center justify-between gap-2"><span className="text-xs">{label}</span><strong>{value}</strong></div><progress className="semantic-progress mt-2" value={value} max={100} aria-label={`${label}: ${value} out of 100`} /></div>
}

function ReportSection({ title, children }: { title: string; children: React.ReactNode }) {
  return <section className="mt-7"><h3 className="mb-3 text-base">{title}</h3><div className="grid gap-3 md:grid-cols-2">{children}</div></section>
}

function ReportItem({ title, tag, children }: { title: string; tag?: string; children: React.ReactNode }) {
  return <article className="rounded-2xl border border-white/[.07] bg-black/15 p-4 text-sm"><div className="flex items-start justify-between gap-3"><h4 className="m-0 text-sm font-semibold capitalize">{title}</h4>{tag && <span className="muted rounded-full bg-white/5 px-2 py-1 text-[10px]">{tag}</span>}</div><div className="muted mt-2 leading-5">{children}</div></article>
}

function Example({ label, value, accent = false }: { label: string; value: string; accent?: boolean }) {
  return <div className="mt-2"><span className="text-[10px] uppercase tracking-wider">{label}</span><div className={accent ? 'mt-1 text-[var(--accent)]' : 'mt-1 text-white'}>{value}</div></div>
}
