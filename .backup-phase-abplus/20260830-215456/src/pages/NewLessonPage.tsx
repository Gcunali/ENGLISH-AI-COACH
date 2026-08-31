import { ArrowLeft, ArrowRight, BriefcaseBusiness, Check, GraduationCap, MessageCircle, Plane, Settings2, Sparkles, Users } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { ErrorState, LoadingState } from '../components/PageState'
import { AppCard, InlineNotice, PageHero, PageShell, SectionHeader } from '../components/ProductUI'
import { useLocalVoiceEngine } from '../hooks/useLocalVoiceEngine'
import { getLessonModes, getStudentLearningProfile } from '../services/native'
import type { LessonDifficulty, LessonFocusArea, LessonModeDefinition, LessonModeId, LessonStartRequest } from '../types'
import { humanize } from '../utils/format'

const ICONS = { free_conversation: MessageCircle, everyday_english: Users, travel_english: Plane, job_interview: BriefcaseBusiness, university_academic: GraduationCap, debate_opinions: Sparkles, custom: Settings2 } as const
const MODE_TAGS: Record<LessonModeId, string[]> = {
  free_conversation: ['Fluent speaking', 'Natural flow'], everyday_english: ['Daily situations', 'Practical phrases'], travel_english: ['Travel', 'Survival phrases'],
  job_interview: ['Professional', 'Interview skills'], university_academic: ['Academic', 'Formal language'], debate_opinions: ['Critical thinking', 'Persuasive speech'], custom: ['Personalized', 'Goal-oriented'],
}
const DIFFICULTY_COPY: Record<LessonDifficulty, string> = { easy: 'More support', standard: 'Balanced', challenging: 'More challenge' }

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
      setModes(availableModes); setDifficulty(profile.defaultLessonDifficulty)
    }).catch((reason: unknown) => setError(reason instanceof Error ? reason.message : String(reason)))
  }
  useEffect(load, [])
  const selected = useMemo(() => modes?.find((item) => item.id === selectedId) ?? null, [modes, selectedId])

  function choose(mode: LessonModeDefinition) {
    setSelectedId(mode.id); setFocusAreas([]); setTopic(''); setObjective(''); setScenario(''); setCustomTitle('')
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
  return <PageShell width="wide">
    <PageHero eyebrow="Guided voice conversation" title="Choose a" accent="lesson mode" description="One consistent teacher, adapted to what you want to practice now." compact back={<Link to="/" className="button-ghost"><ArrowLeft size={16} /> Back to Home</Link>} />

    {!selected && <section aria-label="Lesson modes">
      <SectionHeader title="What would you like to practice?" description="Choose a mode to configure your next local voice lesson." />
      <div className="lesson-mode-grid mt-5">
        {modes!.map((mode) => {
          const Icon = ICONS[mode.id]
          return <button key={mode.id} type="button" onClick={() => choose(mode)} className="lesson-mode-card">
            <span className="lesson-mode-icon"><Icon aria-hidden="true" size={26} /></span>
            <span className="min-w-0 flex-1"><strong>{mode.title}</strong><span className="lesson-mode-description">{mode.description}</span><span className="chip-row">{MODE_TAGS[mode.id].map((tag) => <span className="chip" key={tag}>{tag}</span>)}</span></span>
            <ArrowRight aria-hidden="true" size={18} className="lesson-mode-arrow" />
          </button>
        })}
      </div>
    </section>}

    {selected && <div className="lesson-config-layout">
      <AppCard>
        <button type="button" onClick={() => setSelectedId(null)} className="button-ghost mb-5"><ArrowLeft size={15} /> Choose another mode</button>
        <div className="flex items-start gap-4"><span className="lesson-mode-icon"><SelectedIcon mode={selected.id} /></span><div><h2 className="section-title text-xl">{selected.title}</h2><p className="section-description mt-1">{selected.description}</p><div className="chip-row mt-3">{MODE_TAGS[selected.id].map((tag) => <span className="chip" key={tag}>{tag}</span>)}</div></div></div>
        <fieldset className="mt-8"><legend className="form-label mb-3">Difficulty</legend><div className="difficulty-grid">{selected.supportedDifficulties.map((item) => <button type="button" key={item} aria-pressed={difficulty === item} onClick={() => setDifficulty(item)} className="difficulty-card"><strong>{humanize(item)}</strong><span>{DIFFICULTY_COPY[item]}</span>{difficulty === item && <Check aria-hidden="true" size={15} />}</button>)}</div></fieldset>
        {selected.id === 'custom' && <div className="mt-7 grid gap-4">
          <Field label="Lesson title (optional)" value={customTitle} onChange={setCustomTitle} maxLength={80} />
          <Field label="Topic" value={topic} onChange={setTopic} maxLength={200} required placeholder="Ordering food at a restaurant" />
          <Field label="Objective (optional)" value={objective} onChange={setObjective} maxLength={400} placeholder="Practice speaking naturally with a waiter" multiline />
          <Field label="Scenario (optional)" value={scenario} onChange={setScenario} maxLength={300} placeholder="A casual restaurant" />
          <fieldset><legend className="form-label mb-3">Focus areas (up to 5)</legend><div className="chip-row">{selected.availableFocusAreas.map((area) => <button type="button" key={area} aria-pressed={focusAreas.includes(area)} onClick={() => toggleFocus(area)} className="chip chip-button">{humanize(area)}</button>)}</div></fieldset>
        </div>}
      </AppCard>

      <AppCard as="aside" className="lesson-preview-card">
        <p className="eyebrow">Lesson preview</p>
        <div className="lesson-preview-chat"><span className="lesson-preview-bot"><SelectedIcon mode={selected.id} /></span><p>Hi! I’m your English AI Coach.<br />Let’s practice {customTitle.trim() || selected.title.toLowerCase()}.</p></div>
        {topic.trim() && <div className="lesson-preview-reply">{topic.trim()}</div>}
        <div className="divider" />
        <h2 className="section-title text-base">Configure your lesson</h2>
        <dl className="preview-list"><Preview label="Mode" value={selected.title} /><Preview label="Difficulty" value={humanize(difficulty)} />{topic.trim() && <Preview label="Topic" value={topic.trim()} />}{objective.trim() && <Preview label="Objective" value={objective.trim()} />}{scenario.trim() && <Preview label="Scenario" value={scenario.trim()} />}{focusAreas.length > 0 && <Preview label="Focus" value={focusAreas.map(humanize).join(', ')} />}</dl>
        <button type="button" aria-busy={starting} onClick={() => void start()} disabled={starting || (selected.id === 'custom' && !topic.trim())} className="button-primary mt-6 w-full"><Check size={17} /> {starting ? 'Startingâ€¦' : 'Start Lesson'}</button>
        {selected.id === 'custom' && !topic.trim() && <div className="mt-3"><InlineNotice tone="info">Add a topic to start this custom lesson.</InlineNotice></div>}
      </AppCard>
    </div>}
  </PageShell>
}

function SelectedIcon({ mode }: { mode: LessonModeId }) { const Icon = ICONS[mode]; return <Icon aria-hidden="true" size={25} /> }
function Field({ label, value, onChange, maxLength, required, placeholder, multiline }: { label: string; value: string; onChange: (value: string) => void; maxLength: number; required?: boolean; placeholder?: string; multiline?: boolean }) {
  const id = `lesson-${label.toLowerCase().replace(/[^a-z]+/g, '-')}`
  const common = { value, maxLength, required, placeholder, onChange: (event: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => onChange(event.target.value), className: 'form-control mt-2' }
  return <div><label className="form-label" htmlFor={id}>{label}</label>{multiline ? <textarea {...common} id={id} rows={3} /> : <input {...common} id={id} />}<span className="form-counter">{value.length}/{maxLength}</span></div>
}
function Preview({ label, value }: { label: string; value: string }) { return <div><dt>{label}</dt><dd>{value}</dd></div> }
