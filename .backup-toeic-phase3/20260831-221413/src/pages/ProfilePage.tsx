import { ArrowRight, Save, ShieldCheck, Target } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { ErrorState, LoadingState } from '../components/PageState'
import { getStudentLearningProfile, updateStudentLearningProfile } from '../services/native'
import type { CefrBand, LearningGoal, LessonDifficulty, StudentLearningProfile } from '../types'
import { formatLocalDate, humanize } from '../utils/format'
import { PageShell } from '../components/ProductUI'

const GOALS: { id: LearningGoal; label: string }[] = [
  { id: 'general_fluency', label: 'General Fluency' },
  { id: 'everyday_conversation', label: 'Everyday Conversation' },
  { id: 'travel_english', label: 'Travel English' },
  { id: 'professional_english', label: 'Professional English' },
  { id: 'job_interview', label: 'Job Interview' },
  { id: 'academic_english', label: 'Academic English' },
  { id: 'grammar_accuracy', label: 'Grammar Accuracy' },
  { id: 'vocabulary_growth', label: 'Vocabulary Growth' },
  { id: 'speaking_confidence', label: 'Speaking Confidence' },
  { id: 'exam_preparation', label: 'Exam Preparation' },
]
const LEVELS: CefrBand[] = ['A1', 'A2', 'B1', 'B2', 'C1', 'C2']
const DIFFICULTIES: LessonDifficulty[] = ['easy', 'standard', 'challenging']

export function ProfilePage() {
  const [profile, setProfile] = useState<StudentLearningProfile | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)
  const [saved, setSaved] = useState(false)

  const load = () => {
    setError(null); setProfile(null)
    void getStudentLearningProfile().then(setProfile).catch((reason: unknown) => setError(reason instanceof Error ? reason.message : String(reason)))
  }
  useEffect(load, [])

  const toggleGoal = (goal: LearningGoal) => {
    if (!profile) return
    const selected = profile.learningGoals.includes(goal)
    if (!selected && profile.learningGoals.length >= 3) return
    setSaved(false)
    setProfile({ ...profile, learningGoals: selected ? profile.learningGoals.filter((item) => item !== goal) : [...profile.learningGoals, goal] })
  }
  const save = async () => {
    if (!profile || saving) return
    setSaving(true); setError(null); setSaved(false)
    try {
      const updated = await updateStudentLearningProfile({ targetLevel: profile.targetLevel, learningGoals: profile.learningGoals, defaultLessonDifficulty: profile.defaultLessonDifficulty, useProfileInLessons: profile.useProfileInLessons })
      setProfile(updated); setSaved(true)
      window.dispatchEvent(new Event('student-profile-changed'))
    } catch (reason) { setError(reason instanceof Error ? reason.message : String(reason)) }
    finally { setSaving(false) }
  }

  if (!profile && !error) return <LoadingState label="Loading your local learning profile…" />
  if (!profile && error) return <ErrorState message={error} onRetry={load} />
  const placement = profile!.currentPlacement
  return <PageShell width="standard">
    <header className="mb-7 flex flex-wrap items-start justify-between gap-4"><div><p className="muted mb-2 text-xs uppercase tracking-[.18em]">Student Learning Profile</p><h1 className="m-0 text-2xl md:text-3xl">Your pedagogical preferences</h1><p className="muted text-sm">Placement estimates current ability. You control the target, goals and default challenge.</p></div><div className="flex items-center gap-2 rounded-full border border-white/10 px-3 py-2 text-xs"><ShieldCheck size={14} className="text-[var(--accent)]" /> Stored locally on this device</div></header>
    {error && <div className="mb-5"><ErrorState message={error} onRetry={() => void save()} /></div>}
    <div className="grid gap-5 2xl:grid-cols-2">
      <section className="glass rounded-[28px] p-5 md:p-7"><p className="muted mb-2 text-[10px] uppercase tracking-widest">Current Placement</p><div className="flex items-end gap-3"><strong className="text-5xl text-[var(--accent)]">{placement?.estimatedLevel ?? '—'}</strong><span className="muted pb-1 text-sm">{placement ? 'Estimated CEFR' : 'Not assessed'}</span></div>{placement ? <dl className="mt-5 grid gap-3 sm:grid-cols-2"><Item label="Confidence" value={humanize(placement.confidence)} /><Item label="Last assessed" value={formatLocalDate(placement.assessedAt)} /></dl> : <p className="muted mt-4 text-sm">No completed Placement Test exists. Lessons and scores will not be used to infer a level.</p>}<div className="mt-5 flex flex-wrap gap-3"><Link to="/placement" className="rounded-full bg-[var(--accent)] px-4 py-2 text-sm font-semibold text-[#081006] no-underline">{placement ? 'Retake Placement Test' : 'Take Placement Test'}</Link>{placement && <Link to={`/placement/results/${placement.attemptId}`} className="rounded-full border border-white/15 px-4 py-2 text-sm text-white no-underline">View Placement Result</Link>}</div><p className="muted mb-0 mt-4 text-xs">Your Placement level remains an estimate and cannot be edited here.</p></section>
      <section className="glass rounded-[28px] p-5 md:p-7"><div className="flex items-center gap-2"><Target size={17} className="text-[var(--accent)]" /><h2 className="m-0 text-lg">Target Level</h2></div><p className="muted text-sm">A personal goal, never a replacement for your current estimate.</p><label htmlFor="target-level" className="text-sm">Target CEFR</label><select id="target-level" value={profile!.targetLevel ?? ''} onChange={(event) => { setSaved(false); setProfile({ ...profile!, targetLevel: (event.target.value || null) as CefrBand | null }) }} className="mt-2 w-full rounded-xl border border-white/10 bg-[#0d1119] px-3 py-3 text-white"><option value="">No target</option>{LEVELS.map((level) => <option key={level}>{level}</option>)}</select></section>
      <section className="glass rounded-[28px] p-5 md:p-7"><h2 className="mt-0 text-lg">Learning Goals</h2><p className="muted text-sm">Choose up to 3 explicit goals. Current lesson mode always has priority.</p><div className="flex flex-wrap gap-2" role="group" aria-label="Learning goals">{GOALS.map((goal) => { const active = profile!.learningGoals.includes(goal.id); const disabled = !active && profile!.learningGoals.length >= 3; return <button key={goal.id} type="button" aria-pressed={active} disabled={disabled} onClick={() => toggleGoal(goal.id)} className={`rounded-full border px-3 py-2 text-sm disabled:opacity-35 ${active ? 'border-[var(--accent)] bg-[var(--accent)]/10 text-[var(--accent)]' : 'border-white/10 text-white'}`}>{goal.label}</button> })}</div><p className="muted mb-0 mt-3 text-xs">{profile!.learningGoals.length}/3 selected</p></section>
      <section className="glass rounded-[28px] p-5 md:p-7"><h2 className="mt-0 text-lg">Default Lesson Difficulty</h2><p className="muted text-sm">This sets the initial difficulty when creating a new lesson. You can still change it for each lesson.</p><div className="flex flex-wrap gap-2" role="group" aria-label="Default lesson difficulty">{DIFFICULTIES.map((difficulty) => <button key={difficulty} type="button" aria-pressed={profile!.defaultLessonDifficulty === difficulty} onClick={() => { setSaved(false); setProfile({ ...profile!, defaultLessonDifficulty: difficulty }) }} className={`rounded-full border px-4 py-2 text-sm ${profile!.defaultLessonDifficulty === difficulty ? 'border-[var(--accent)] bg-[var(--accent)]/10 text-[var(--accent)]' : 'border-white/10 text-white'}`}>{humanize(difficulty)}</button>)}</div></section>
    </div>
    <section className="glass mt-5 rounded-[28px] p-5 md:p-7"><label className="flex cursor-pointer items-start gap-3"><input type="checkbox" checked={profile!.useProfileInLessons} onChange={(event) => { setSaved(false); setProfile({ ...profile!, useProfileInLessons: event.target.checked }) }} className="mt-1" /><span><strong className="block text-sm">Use Student Profile in Lessons</strong><span className="muted mt-1 block text-xs">Uses your Placement level and learning goals to adapt future conversations. Learning Memory has its own independent setting.</span></span></label></section>
    <div className="mt-5 flex items-center justify-end gap-3">{saved && <span role="status" className="text-sm text-[var(--accent)]">Profile saved locally.</span>}<button type="button" disabled={saving} onClick={() => void save()} className="flex items-center gap-2 rounded-full bg-[var(--accent)] px-5 py-3 font-semibold text-[#081006] disabled:opacity-50"><Save size={16} /> {saving ? 'Saving…' : 'Save Profile'}</button><Link to="/lesson/new" className="muted flex items-center gap-1 text-sm no-underline">New lesson <ArrowRight size={14} /></Link></div>
  </PageShell>
}

function Item({ label, value }: { label: string; value: string }) { return <div className="rounded-xl bg-white/[.035] p-3"><div className="font-semibold text-sm">{value}</div><div className="muted mt-1 text-[10px]">{label}</div></div> }
