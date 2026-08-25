import { ArrowLeft, BookOpenCheck, Clock3, GraduationCap, Play, RotateCcw } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { EmptyState, ErrorState, LoadingState } from '../components/PageState'
import { InlineNotice, PageHeader, PageShell, SectionHeader, StatusBadge } from '../components/ProductUI'
import { getCourseCatalog, startGuidedLesson } from '../services/native'
import type { Course, CourseLesson, CurriculumCatalog, CurriculumProgress } from '../types'

export function CoursePage() {
  const { curriculumId, levelId, unitId } = useParams()
  const navigate = useNavigate()
  const [catalog, setCatalog] = useState<CurriculumCatalog>()
  const [error, setError] = useState<string | null>(null)
  const [busyLesson, setBusyLesson] = useState<string | null>(null)
  const load = useCallback(async () => {
    setError(null)
    try { setCatalog(await getCourseCatalog()) }
    catch (value) { setError(value instanceof Error ? value.message : String(value)) }
  }, [])
  useEffect(() => { void load() }, [load])
  if (!catalog && !error) return <LoadingState label="Loading course content…" />
  if (error && !catalog) return <ErrorState message={error} onRetry={() => void load()} />
  if (!catalog || catalog.curricula.length === 0) return <PageShell><PageHeader eyebrow="English Course" title="Course" description="Structured local Guided Lesson paths."/><EmptyState title="No course content is installed yet." message="Course content will appear here after a valid published local Curriculum is installed. Guided Lessons remain available separately." action={<Link className="button-secondary inline-flex no-underline" to="/guided-lessons">Open Guided Lessons</Link>}/></PageShell>

  if (!curriculumId && catalog.curricula.length > 1) return <PageShell width="wide"><PageHeader eyebrow="English Course" title="Choose a course" description="Each course is local, versioned content."/><div className="grid gap-4 md:grid-cols-2">{catalog.curricula.map(course => <CourseCard key={course.curriculumId} course={course}/>)}</div></PageShell>
  const course = curriculumId ? catalog.curricula.find(item => item.curriculumId === curriculumId) : catalog.curricula[0]
  if (!course) return <NotAvailable back="/course" label="Course" />
  const level = levelId ? course.levels.find(item => item.levelId === levelId) : undefined
  if (levelId && !level) return <NotAvailable back={`/course/${course.curriculumId}`} label="Level" />
  const unit = unitId ? level?.units.find(item => item.unitId === unitId) : undefined
  if (unitId && !unit) return <NotAvailable back={`/course/${course.curriculumId}/${levelId}`} label="Unit" />

  const start = async (lesson: CourseLesson) => {
    setBusyLesson(lesson.lessonId); setError(null)
    try {
      const session = await startGuidedLesson(lesson.lessonId, false, lesson.contentVersion)
      navigate(`/guided-lessons/session/${session.id}`)
    } catch (value) { setError(value instanceof Error ? value.message : String(value)) }
    finally { setBusyLesson(null) }
  }

  if (unit && level) return <PageShell width="wide"><Back to={`/course/${course.curriculumId}/${level.levelId}`} label={level.cefrLevel}/><PageHeader eyebrow={`${course.title} · ${level.cefrLevel}`} title={unit.title} description={unit.description}/><Progress value={unit.progress} label={`${unit.title} progress`}/><Metadata title="Unit objectives" values={unit.objectives}/><Metadata title="Skill focus" values={unit.skillFocus.map(humanize)}/><Metadata title="Grammar topics" values={unit.grammarTopics}/><Metadata title="Vocabulary topics" values={unit.vocabularyTopics}/><Metadata title="Communicative functions" values={unit.communicativeFunctions}/><SectionHeader title="Lessons" description="Recommended order. Every lesson remains freely accessible."/>{error&&<InlineNotice tone="error" live>{error}</InlineNotice>}<div className="grid gap-4 lg:grid-cols-2">{unit.lessons.map(lesson=><LessonCard key={lesson.lessonId} lesson={lesson} catalog={catalog} busy={busyLesson===lesson.lessonId} start={()=>void start(lesson)}/>)}</div></PageShell>
  if (level) return <PageShell width="wide"><Back to={`/course/${course.curriculumId}`} label={course.title}/><PageHeader eyebrow="Course level" title={`${level.cefrLevel} · ${level.title}`} description={level.description}/>{level.recommended&&<InlineNotice title="Suggested from your placement">This is a starting-point recommendation, never an access restriction.</InlineNotice>}{level.target&&<InlineNotice title="Your target level">Your target is contextual only and does not change progress.</InlineNotice>}<Progress value={level.progress} label={`${level.cefrLevel} level progress`}/><Metadata title="Level objectives" values={level.objectives}/><SectionHeader title="Units" description="Open any unit in the recommended sequence."/><div className="grid gap-4 md:grid-cols-2">{level.units.map(unitItem=><article key={unitItem.unitId} className="glass min-w-0 rounded-[24px] p-5"><h3 className="mt-0 break-words text-lg">{unitItem.title}</h3><p className="muted break-words text-sm">{unitItem.description}</p><div className="flex flex-wrap gap-2">{unitItem.skillFocus.map(skill=><StatusBadge key={skill}>{humanize(skill)}</StatusBadge>)}</div><Progress value={unitItem.progress} label={`${unitItem.title} progress`}/><Link className="button-secondary mt-4 inline-flex no-underline" to={`/course/${course.curriculumId}/${level.levelId}/${unitItem.unitId}`}>View Unit</Link></article>)}</div></PageShell>

  const placementOutsideInstalledCourse = course.suggestedLevel && !course.levels.some(item=>item.cefrLevel===course.suggestedLevel)
  const installedRange = course.levels.map(item=>item.cefrLevel).join('–')
  return <PageShell width="wide"><PageHeader eyebrow="English Course" title={course.title} description={course.description}/><div className="grid gap-4 md:grid-cols-3"><Summary label="Course progress" value={`${course.progress.completedLessons} / ${course.progress.totalLessons}`}/><Summary label="Suggested starting point" value={course.suggestedLevel ?? 'Choose a level'} detail={course.suggestedLevel?'From your latest Placement Test':'Take the Placement Test for a suggestion'}/><Summary label="Target level" value={course.targetLevel ?? 'Not set'} detail="A goal, never a lock"/></div><Progress value={course.progress} label={`${course.title} progress`}/>{!course.suggestedLevel&&<InlineNotice title="Choose a level"><span>No Placement result is available, so Course does not silently recommend A1. </span><Link to="/placement">Take the placement test</Link><span> or freely choose any level.</span></InlineNotice>}{placementOutsideInstalledCourse&&<InlineNotice title="Placement level is outside the installed Course"><span>Your Placement result is {course.suggestedLevel}. This installed Course currently includes {installedRange}, so no equivalent Course level is marked as suggested. Your Placement result is unchanged.</span></InlineNotice>}<SectionHeader title="Levels" description="CEFR labels classify content. They do not classify or restrict you."/><div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">{course.levels.map(item=><article key={item.levelId} className="glass min-w-0 rounded-[24px] p-5"><div className="flex flex-wrap gap-2"><StatusBadge tone="info">{item.cefrLevel}</StatusBadge>{item.recommended&&<StatusBadge tone="success">Suggested from your placement</StatusBadge>}{item.target&&<StatusBadge>Your target</StatusBadge>}</div><h3 className="mt-4 break-words text-lg">{item.title}</h3><p className="muted break-words text-sm">{item.description}</p><Progress value={item.progress} label={`${item.cefrLevel} progress`}/><Link className="button-secondary mt-4 inline-flex no-underline" to={`/course/${course.curriculumId}/${item.levelId}`}>Explore {item.cefrLevel}</Link></article>)}</div><p className="muted mt-6 text-sm"><GraduationCap className="mr-2 inline" size={16}/>Course organizes Guided Lessons. It does not replace the Guided Lessons library.</p></PageShell>
}

function LessonCard({lesson,catalog,busy,start}:{lesson:CourseLesson;catalog:CurriculumCatalog;busy:boolean;start:()=>void}) {
  const active = catalog.activeSession
  const anotherActive = Boolean(active && active.lessonId !== lesson.lessonId)
  const status = lesson.status === 'not_started' ? 'Not started' : lesson.status === 'in_progress' ? 'In progress' : 'Completed'
  return <article className="glass min-w-0 rounded-[24px] p-5"><div className="flex flex-wrap gap-2"><StatusBadge tone={lesson.status==='completed'?'success':lesson.status==='in_progress'?'warning':'neutral'}>{status}</StatusBadge>{lesson.hasUpdatedContentAvailable&&<StatusBadge tone="info">Updated content available</StatusBadge>}</div><h3 className="mt-4 break-words text-lg">{lesson.title}</h3><p className="muted break-words text-sm">{lesson.description}</p><div className="muted flex flex-wrap gap-3 text-xs"><span>{lesson.cefrBand} content</span><span><Clock3 className="mr-1 inline" size={13}/>{lesson.estimatedMinutes} min</span><span>Version {lesson.contentVersion}</span></div><div className="mt-5 flex flex-wrap gap-3">{lesson.status==='in_progress'&&lesson.activeSessionId?<Link className="button-primary inline-flex no-underline" to={`/guided-lessons/session/${lesson.activeSessionId}`}><Play size={15}/>Resume Lesson</Link>:<button className="button-primary" disabled={!lesson.startable||busy||anotherActive} onClick={start}>{lesson.status==='completed'?<RotateCcw size={15}/>:<Play size={15}/>} {lesson.status==='completed'?(lesson.hasUpdatedContentAvailable?'Review Update':'Review Lesson'):'Start Lesson'}</button>}{anotherActive&&active&&<Link className="button-secondary inline-flex no-underline" to={`/guided-lessons/session/${active.sessionId}`}>Resume existing lesson</Link>}</div>{!lesson.startable&&<p role="status" className="mt-3 text-sm">This exact lesson version is not runtime-ready.</p>}{anotherActive&&<p className="muted mt-3 text-xs">The existing Guided Lesson must be resumed or explicitly abandoned before starting another one.</p>}</article>
}
function CourseCard({course}:{course:Course}) {return <article className="glass rounded-[24px] p-5"><BookOpenCheck className="text-[var(--accent)]"/><h2 className="section-title mt-4">{course.title}</h2><p className="muted text-sm">{course.description}</p><Progress value={course.progress} label={`${course.title} progress`}/><Link className="button-secondary mt-4 inline-flex no-underline" to={`/course/${course.curriculumId}`}>Open Course</Link></article>}
function Progress({value,label}:{value:CurriculumProgress;label:string}) {return <div className="mt-4"><div className="mb-2 flex flex-wrap justify-between gap-2 text-sm"><span>{value.completedLessons} / {value.totalLessons} Lessons completed</span><strong>{value.percent}%</strong></div><div role="progressbar" aria-label={label} aria-valuemin={0} aria-valuemax={100} aria-valuenow={value.percent} className="h-2 overflow-hidden rounded-full bg-white/[.08]"><div className="h-full rounded-full bg-[var(--accent)]" style={{width:`${value.percent}%`}}/></div></div>}
function Summary({label,value,detail}:{label:string;value:string;detail?:string}) {return <div className="metric-card"><div className="metric-label">{label}</div><div className="metric-value break-words">{value}</div>{detail&&<div className="metric-detail">{detail}</div>}</div>}
function Metadata({title,values}:{title:string;values:string[]}) {if(!values.length)return null;return <section className="mt-5"><h2 className="section-title">{title}</h2><ul className="flex flex-wrap gap-2 p-0">{values.map(value=><li key={value} className="status-badge break-words">{value}</li>)}</ul></section>}
function Back({to,label}:{to:string;label:string}) {return <Link to={to} className="muted mb-5 inline-flex items-center gap-2 text-sm no-underline"><ArrowLeft size={15}/>{label}</Link>}
function NotAvailable({back,label}:{back:string;label:string}) {return <PageShell><PageHeader title={`${label} not available`} description="This local Course item is unavailable or invalid."/><Back to={back} label="Back to Course"/></PageShell>}
function humanize(value:string){return value.replaceAll('_',' ').replace(/\b\w/g,letter=>letter.toUpperCase())}
