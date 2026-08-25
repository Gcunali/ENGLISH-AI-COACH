import { ArrowLeft, BriefcaseBusiness, Check, GraduationCap, MessageCircle, Plane, Settings2, Sparkles, Users } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { ErrorState, LoadingState } from '../components/PageState'
import { InlineNotice, PageHeader, PageShell } from '../components/ProductUI'
import { useLocalVoiceEngine } from '../hooks/useLocalVoiceEngine'
import { getLessonModes, getStudentLearningProfile } from '../services/native'
import type { LessonDifficulty, LessonFocusArea, LessonModeDefinition, LessonModeId, LessonStartRequest } from '../types'
import { humanize } from '../utils/format'

const ICONS = { free_conversation: MessageCircle, everyday_english: Users, travel_english: Plane, job_interview: BriefcaseBusiness, university_academic: GraduationCap, debate_opinions: Sparkles, custom: Settings2 } as const
const DIFFICULTY_COPY: Record<LessonDifficulty, string> = { easy: 'More support and a gentler pace.', standard: 'Balanced challenge for regular practice.', challenging: 'Less support and more demanding language.' }

export function NewLessonPage() {
  const [modes, setModes] = useState<LessonModeDefinition[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [selectedId, setSelectedId] = useState<LessonModeId | null>(null)
  const [difficulty, setDifficulty] = useState<LessonDifficulty>('standard')
  const [topic, setTopic] = useState('')
  const [objective, setObjective] = useState('')
  const [scenario, setScenario] = useState('')
  const [customTitle, setCustomTitle] = useState('')
  const [focusAreas, setFocusAreas] = useState<LessonFocusArea[]>([])
  const [starting, setStarting] = useState(false)
  const voice = useLocalVoiceEngine()
  const navigate = useNavigate()

  const load = () => {
    setError(null); setModes(null)
    void Promise.all([getLessonModes(), getStudentLearningProfile()]).then(([availableModes, profile]) => {
      setModes(availableModes)
      setDifficulty(profile.defaultLessonDifficulty)
    }).catch((reason: unknown) => setError(reason instanceof Error ? reason.message : String(reason)))
  }
  useEffect(load, [])
  const selected = useMemo(() => modes?.find((item) => item.id === selectedId) ?? null, [modes, selectedId])

  function choose(mode: LessonModeDefinition) {
    setSelectedId(mode.id); setFocusAreas([])
    setTopic(''); setObjective(''); setScenario(''); setCustomTitle('')
  }
  function toggleFocus(area: LessonFocusArea) {
    setFocusAreas((current) => current.includes(area) ? current.filter((item) => item !== area) : current.length < 5 ? [...current, area] : current)
  }
  async function start() {
    if (!selected || starting || (selected.id === 'custom' && !topic.trim())) return
    const request: LessonStartRequest = { modeId: selected.id, difficulty, focusAreas }
    if (topic.trim()) request.topic = topic
    if (objective.trim()) request.objective = objective
    if (scenario.trim()) request.scenario = scenario
    if (customTitle.trim()) request.customTitle = customTitle
    setStarting(true)
    const started = await voice.startLesson(request)
    setStarting(false)
    if (started) navigate('/')
  }

  if (!modes && !error) return <LoadingState label="Loading lesson modes from the local registry…" />
  if (error) return <ErrorState message={error} onRetry={load} />
  return <PageShell width="standard">
    <PageHeader eyebrow="Guided voice conversation" title="Choose a lesson mode" description="One consistent teacher, adapted to what you want to practice now." back={<Link to="/" className="button-ghost"><ArrowLeft size={15} /> Back to Home</Link>} />
    {!selected && <section className="grid gap-4 md:grid-cols-2 2xl:grid-cols-3" aria-label="Lesson modes">
      {modes!.map((mode) => { const Icon = ICONS[mode.id]; return <button key={mode.id} type="button" onClick={() => choose(mode)} className="glass min-h-44 rounded-2xl p-5 text-left text-white transition hover:border-[var(--accent)]/40"><Icon aria-hidden="true" size={21} className="text-[var(--accent)]" /><h2 className="mb-2 mt-4 text-base">{mode.title}</h2><p className="muted m-0 text-sm leading-6">{mode.description}</p></button> })}
    </section>}
    {selected && <div className="grid gap-5 2xl:grid-cols-[1fr_360px]">
      <section className="glass rounded-[28px] p-5 md:p-7">
        <button type="button" onClick={() => setSelectedId(null)} className="muted mb-5 inline-flex items-center gap-2 text-sm"><ArrowLeft size={14} /> Choose another mode</button>
        <h2 className="mt-0 text-xl">{selected.title}</h2><p className="muted text-sm">{selected.description}</p>
        <fieldset className="mt-6"><legend className="mb-3 text-sm font-medium">Difficulty</legend><div className="grid gap-2 sm:grid-cols-3">{selected.supportedDifficulties.map((item) => <button type="button" key={item} aria-pressed={difficulty === item} onClick={() => setDifficulty(item)} className={`rounded-xl border p-3 text-left text-sm ${difficulty === item ? 'border-[var(--accent)] bg-[var(--accent)]/10 text-[var(--accent)]' : 'border-white/10 text-white'}`}><strong className="block">{humanize(item)}</strong><span className="muted mt-1 block text-xs">{DIFFICULTY_COPY[item]}</span></button>)}</div></fieldset>
        {selected.id === 'custom' && <div className="mt-6 grid gap-4">
          <Field label="Lesson title (optional)" value={customTitle} onChange={setCustomTitle} maxLength={80} />
          <Field label="Topic" value={topic} onChange={setTopic} maxLength={200} required placeholder="Ordering food at a restaurant" />
          <Field label="Objective (optional)" value={objective} onChange={setObjective} maxLength={400} placeholder="Practice speaking naturally with a waiter" multiline />
          <Field label="Scenario (optional)" value={scenario} onChange={setScenario} maxLength={300} placeholder="A casual restaurant" />
          <fieldset><legend className="mb-3 text-sm font-medium">Focus areas (up to 5)</legend><div className="flex flex-wrap gap-2">{selected.availableFocusAreas.map((area) => <button type="button" key={area} aria-pressed={focusAreas.includes(area)} onClick={() => toggleFocus(area)} className={`rounded-full border px-3 py-2 text-xs ${focusAreas.includes(area) ? 'border-[var(--accent)] bg-[var(--accent)]/10 text-[var(--accent)]' : 'border-white/10 text-white'}`}>{humanize(area)}</button>)}</div></fieldset>
        </div>}
      </section>
      <aside className="glass h-fit rounded-[28px] p-5"><p className="eyebrow">Lesson preview</p><h2 className="section-title text-lg">{customTitle.trim() || selected.title}</h2><dl className="space-y-3 text-sm"><Preview label="Mode" value={selected.title} /><Preview label="Difficulty" value={humanize(difficulty)} />{topic.trim() && <Preview label="Topic" value={topic.trim()} />}{objective.trim() && <Preview label="Objective" value={objective.trim()} />}{scenario.trim() && <Preview label="Scenario" value={scenario.trim()} />}{focusAreas.length > 0 && <Preview label="Focus" value={focusAreas.map(humanize).join(', ')} />}</dl><p className="muted mt-5 text-xs">The internal teaching context is validated and stays hidden. Starting creates the lesson snapshot.</p><button type="button" aria-busy={starting} onClick={() => void start()} disabled={starting || (selected.id === 'custom' && !topic.trim())} className="button-primary mt-4 w-full"><Check size={17} /> {starting ? 'Starting…' : 'Start Lesson'}</button>{selected.id === 'custom' && !topic.trim() && <div className="mt-3"><InlineNotice tone="info">Add a topic to start this custom lesson.</InlineNotice></div>}</aside>
    </div>}
  </PageShell>
}

function Field({ label, value, onChange, maxLength, required, placeholder, multiline }: { label: string; value: string; onChange: (value: string) => void; maxLength: number; required?: boolean; placeholder?: string; multiline?: boolean }) {
  const id = `lesson-${label.toLowerCase().replace(/[^a-z]+/g, '-')}`
  const common = { value, maxLength, required, placeholder, onChange: (event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => onChange(event.target.value), className: 'mt-2 w-full rounded-xl border border-white/10 bg-[#0d1119] px-3 py-3 text-sm text-white' }
  return <div className="text-sm"><label htmlFor={id}>{label}</label>{multiline ? <textarea {...common} id={id} rows={3} /> : <input {...common} id={id} />}<span className="muted mt-1 block text-right text-[10px]">{value.length}/{maxLength}</span></div>
}
function Preview({ label, value }: { label: string; value: string }) { return <div><dt className="muted text-[10px] uppercase tracking-wider">{label}</dt><dd className="m-0 mt-1 whitespace-pre-wrap break-words">{value}</dd></div> }
