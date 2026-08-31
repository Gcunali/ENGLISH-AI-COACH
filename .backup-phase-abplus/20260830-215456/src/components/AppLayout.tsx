import { Activity, Award, BookOpen, ChevronDown, CircleGauge, ClipboardCheck, GraduationCap, History, Home, Mic2, RefreshCw, Settings, Sparkles, UserRound } from 'lucide-react'
import { NavLink, Outlet, useLocation } from 'react-router-dom'
import { useEffect, useState } from 'react'
import { getCourseCatalog, getGuidedLessonOverview, probeLocalVoiceEngine, subscribeGamificationChanges, subscribeReviewChanges } from '../services/native'
import type { Achievement } from '../types'
import coachMascot from '../assets/coach-mascot-v1.png'
import { notifyGamificationDataChanged } from '../utils/gamificationData'
import { notifyReviewDataChanged } from '../utils/reviewData'
import { RouteErrorBoundary } from './RouteErrorBoundary'

const NAVIGATION = [
  { label: 'Practice', items: [
    { to: '/', label: 'Home', icon: Home, end: true },
    { to: '/lesson/new', label: 'New Lesson', icon: Sparkles },
    { to: '/review', label: 'Review', icon: RefreshCw },
    { to: '/pronunciation', label: 'Pronunciation', icon: Mic2 },
  ] },
  { label: 'Learning', items: [
    { to: '/course', label: 'Course', icon: BookOpen, course: true },
    { to: '/guided-lessons', label: 'Guided Lessons', icon: GraduationCap, guided: true },
    { to: '/vocabulary', label: 'Vocabulary', icon: BookOpen },
    { to: '/progress', label: 'Progress', icon: CircleGauge },
    { to: '/history', label: 'History', icon: History },
  ] },
  { label: 'Assessment & profile', items: [
    { to: '/placement', label: 'Placement Test', icon: ClipboardCheck },
    { to: '/profile', label: 'Student Profile', icon: UserRound },
    { to: '/achievements', label: 'Achievements', icon: Award },
  ] },
  { label: 'System', items: [
    { to: '/settings', label: 'Settings', icon: Settings },
    { to: '/diagnostics', label: 'Diagnostics', icon: Activity },
  ] },
] as const

const ROUTE_TITLES: Array<[RegExp, string]> = [
  [/^\/$/, 'Home'], [/^\/lesson\/new$/, 'New Lesson'], [/^\/history\//, 'Lesson Details'], [/^\/history$/, 'History'],
  [/^\/guided-lessons\/session\//, 'Guided Lesson'], [/^\/guided-lessons\//, 'Guided Lesson Details'], [/^\/guided-lessons$/, 'Guided Lessons'],
  [/^\/course/, 'English Course'],
  [/^\/progress$/, 'Progress'], [/^\/review\/session\//, 'Review Session'], [/^\/review$/, 'Review'],
  [/^\/placement\/results\//, 'Placement Result'], [/^\/placement$/, 'Placement Test'], [/^\/profile$/, 'Student Profile'],
  [/^\/achievements$/, 'Achievements'], [/^\/vocabulary\//, 'Vocabulary Details'], [/^\/vocabulary$/, 'Vocabulary'],
  [/^\/pronunciation$/, 'Pronunciation Practice'], [/^\/settings$/, 'Settings'], [/^\/diagnostics$/, 'System Diagnostics'],
]

export function AppLayout() {
  const { pathname } = useLocation()
  const [unlocked, setUnlocked] = useState<Achievement | null>(null)
  const [showGuidedLessons, setShowGuidedLessons] = useState(false)
  const [showCourse, setShowCourse] = useState(false)
  const [localAiReady, setLocalAiReady] = useState<boolean | null>(null)
  useEffect(() => {
    document.documentElement.scrollTop = 0
    document.body.scrollTop = 0
    const title = ROUTE_TITLES.find(([pattern]) => pattern.test(pathname))?.[1] ?? 'Page not found'
    document.title = `${title} - English AI Coach`
  }, [pathname])
  useEffect(() => {
    let stop: () => void = () => undefined
    void subscribeGamificationChanges((result) => {
      notifyGamificationDataChanged()
      if (result?.achievementsUnlocked.length) setUnlocked(result.achievementsUnlocked[0])
    }).then((unlisten) => { stop = unlisten })
    return () => stop()
  }, [])
  useEffect(()=>{let stop:()=>void=()=>undefined;void subscribeReviewChanges(notifyReviewDataChanged).then(value=>{stop=value});return()=>stop()},[])
  useEffect(()=>{let active=true;void Promise.all([getGuidedLessonOverview(),getCourseCatalog()]).then(([guided,course])=>{if(active){setShowGuidedLessons(guided.publishedLessonCount>0||Boolean(guided.activeSession));setShowCourse(course.publishedCurriculumCount>0)}}).catch(()=>{if(active){setShowGuidedLessons(false);setShowCourse(false)}});return()=>{active=false}},[pathname])
  useEffect(()=>{let active=true;void probeLocalVoiceEngine().then(value=>{if(active)setLocalAiReady(value.offlineReady)}).catch(()=>{if(active)setLocalAiReady(false)});return()=>{active=false}},[])
  useEffect(() => { if (!unlocked) return undefined; const timer = window.setTimeout(() => setUnlocked(null), 5000); return () => window.clearTimeout(timer) }, [unlocked])
  return <div className="app-shell flex min-h-screen">
    <a className="skip-link" href="#main-content">Skip to main content</a>
    <aside className="desktop-nav app-sidebar h-screen w-64 shrink-0 overflow-y-auto p-5 flex flex-col">
      <Brand />
      <Navigation showGuidedLessons={showGuidedLessons} showCourse={showCourse} />
      <ProfileCard />
    </aside>
    <div className="app-content min-w-0 flex-1">
      <div className="mobile-nav p-3">
        <div className="flex items-center justify-between gap-3"><Brand /><LocalAiStatus ready={localAiReady} /></div>
        <Navigation compact showGuidedLessons={showGuidedLessons} showCourse={showCourse} />
      </div>
      <main id="main-content" tabIndex={-1} className="app-main mx-auto w-full min-w-0 overflow-x-hidden"><div className="app-utility-bar"><LocalAiStatus ready={localAiReady} /></div><RouteErrorBoundary key={pathname}><Outlet /></RouteErrorBoundary></main>
    </div>
    {unlocked && <div role="status" className="fixed bottom-5 right-5 z-50 max-w-sm rounded-2xl border border-[var(--accent)]/25 bg-white p-4 shadow-2xl"><div className="flex items-center gap-2 text-sm font-semibold"><Award size={18} className="text-[var(--accent)]" /> Achievement unlocked</div><p className="mb-0 mt-1 text-sm">{unlocked.title}</p></div>}
  </div>
}

function Brand() {
  return <div className="app-brand flex items-center gap-3 px-2 py-2">
    <div className="brand-mark"><img src={coachMascot} alt="" /></div>
    <div><div className="font-bold tracking-tight">English AI</div><div className="text-xs muted">Local Coach</div></div>
  </div>
}

function Navigation({ compact = false, showGuidedLessons = false, showCourse = false }: { compact?: boolean; showGuidedLessons?: boolean; showCourse?: boolean }) {
  return <nav className={compact ? 'scrollbar-none mt-3 flex gap-2 overflow-x-auto pb-1' : 'mt-5 text-sm'} aria-label="Main navigation">
    {NAVIGATION.map((group) => <div key={group.label} className={compact ? 'contents' : undefined}>{!compact && <div className="nav-group-label">{group.label}</div>}{group.items.filter(item=>(!('guided' in item)||showGuidedLessons)&&(!('course' in item)||showCourse)).map(({ to, label, icon: Icon, ...options }) => <NavLink
      key={to}
      to={to}
      end={'end' in options ? options.end : false}
      className={({ isActive }) => `nav-link ${compact ? 'shrink-0' : 'mb-1 w-full'} flex items-center gap-3 rounded-xl px-3 py-2.5 no-underline ${isActive ? 'is-active' : ''}`}
    ><Icon aria-hidden="true" size={17} /><span>{label}</span></NavLink>)}</div>)}
  </nav>
}

function LocalAiStatus({ ready }: { ready: boolean | null }) {
  const label = ready === null ? 'Checking' : ready ? 'Ready' : 'Needs setup'
  return <div className={`local-ai-status ${ready === false ? 'local-ai-status-warning' : ''}`} role="status"><span className="local-ai-dot" aria-hidden="true" />Local AI <span aria-hidden="true">·</span> {label}<ChevronDown aria-hidden="true" size={14}/></div>
}

function ProfileCard() {
  return <div className="sidebar-profile mt-auto"><div className="sidebar-avatar"><UserRound aria-hidden="true" size={20}/></div><div className="min-w-0 flex-1"><div className="truncate text-sm font-semibold">Local learner</div><div className="truncate text-[10px] muted">Private profile</div></div><ChevronDown aria-hidden="true" size={14} className="muted"/></div>
}
