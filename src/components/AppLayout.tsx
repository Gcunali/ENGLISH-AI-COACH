import { Activity, Award, BookOpen, CircleGauge, ClipboardCheck, GraduationCap, History, Home, Mic2, RefreshCw, Settings, ShieldCheck, Sparkles, UserRound } from 'lucide-react'
import { NavLink, Outlet, useLocation } from 'react-router-dom'
import { useEffect, useState } from 'react'
import { getGuidedLessonOverview, subscribeGamificationChanges, subscribeReviewChanges } from '../services/native'
import type { Achievement } from '../types'
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
  [/^\/progress$/, 'Progress'], [/^\/review\/session\//, 'Review Session'], [/^\/review$/, 'Review'],
  [/^\/placement\/results\//, 'Placement Result'], [/^\/placement$/, 'Placement Test'], [/^\/profile$/, 'Student Profile'],
  [/^\/achievements$/, 'Achievements'], [/^\/vocabulary\//, 'Vocabulary Details'], [/^\/vocabulary$/, 'Vocabulary'],
  [/^\/pronunciation$/, 'Pronunciation Practice'], [/^\/settings$/, 'Settings'], [/^\/diagnostics$/, 'System Diagnostics'],
]

export function AppLayout() {
  const { pathname } = useLocation()
  const [unlocked, setUnlocked] = useState<Achievement | null>(null)
  const [showGuidedLessons, setShowGuidedLessons] = useState(false)
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
  useEffect(()=>{let active=true;void getGuidedLessonOverview().then(value=>{if(active)setShowGuidedLessons(value.publishedLessonCount>0||Boolean(value.activeSession))}).catch(()=>{if(active)setShowGuidedLessons(false)});return()=>{active=false}},[pathname])
  useEffect(() => { if (!unlocked) return undefined; const timer = window.setTimeout(() => setUnlocked(null), 5000); return () => window.clearTimeout(timer) }, [unlocked])
  return <div className="app-shell flex min-h-screen">
    <a className="skip-link" href="#main-content">Skip to main content</a>
    <aside className="desktop-nav h-screen w-64 shrink-0 overflow-y-auto border-r border-white/[.07] p-5 flex flex-col">
      <Brand />
      <Navigation showGuidedLessons={showGuidedLessons} />
      <div className="mt-auto glass rounded-2xl p-4">
        <div className="flex items-center gap-2 text-sm font-medium"><ShieldCheck size={16} className="text-[var(--accent)]" /> Private by default</div>
        <p className="muted text-xs leading-5 mb-0">Audio, transcripts and learning data stay on this computer.</p>
      </div>
    </aside>
    <div className="min-w-0 flex-1">
      <div className="mobile-nav border-b border-white/[.07] p-3">
        <Brand />
        <Navigation compact showGuidedLessons={showGuidedLessons} />
      </div>
      <main id="main-content" tabIndex={-1} className="mx-auto w-full min-w-0 max-w-[1500px] overflow-x-hidden p-5 md:p-8"><RouteErrorBoundary key={pathname}><Outlet /></RouteErrorBoundary></main>
    </div>
    {unlocked && <div role="status" className="fixed bottom-5 right-5 z-50 max-w-sm rounded-2xl border border-[var(--accent)]/25 bg-[#151b24] p-4 shadow-2xl"><div className="flex items-center gap-2 text-sm font-semibold"><Award size={18} className="text-[var(--accent)]" /> Achievement unlocked</div><p className="mb-0 mt-1 text-sm">{unlocked.title}</p></div>}
  </div>
}

function Brand() {
  return <div className="flex items-center gap-3 px-2 py-2">
    <div className="h-9 w-9 rounded-xl bg-[var(--accent)] text-black grid place-items-center"><Sparkles size={18} /></div>
    <div><div className="font-semibold tracking-tight">English AI</div><div className="text-xs muted">Local Coach</div></div>
  </div>
}

function Navigation({ compact = false, showGuidedLessons = false }: { compact?: boolean; showGuidedLessons?: boolean }) {
  return <nav className={compact ? 'scrollbar-none mt-3 flex gap-2 overflow-x-auto pb-1' : 'mt-5 text-sm'} aria-label="Main navigation">
    {NAVIGATION.map((group) => <div key={group.label} className={compact ? 'contents' : undefined}>{!compact && <div className="nav-group-label">{group.label}</div>}{group.items.filter(item=>!('guided' in item)||showGuidedLessons).map(({ to, label, icon: Icon, ...options }) => <NavLink
      key={to}
      to={to}
      end={'end' in options ? options.end : false}
      className={({ isActive }) => `nav-link ${compact ? 'shrink-0' : 'mb-1 w-full'} flex items-center gap-2 rounded-xl px-3 py-2.5 no-underline ${isActive ? 'bg-white/[.08] text-white' : 'muted'}`}
    ><Icon aria-hidden="true" size={17} /><span>{label}</span></NavLink>)}</div>)}
  </nav>
}
