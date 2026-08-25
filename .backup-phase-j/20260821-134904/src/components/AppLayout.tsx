import { BookOpen, CircleGauge, History, Home, Settings, ShieldCheck, Sparkles } from 'lucide-react'
import { NavLink, Outlet } from 'react-router-dom'

const NAVIGATION = [
  { to: '/', label: 'Home', icon: Home, end: true },
  { to: '/history', label: 'History', icon: History },
  { to: '/progress', label: 'Progress', icon: CircleGauge },
  { to: '/vocabulary', label: 'Vocabulary', icon: BookOpen },
  { to: '/settings', label: 'Settings', icon: Settings },
] as const

export function AppLayout() {
  return <div className="app-shell flex min-h-screen">
    <aside className="desktop-nav w-64 shrink-0 border-r border-white/[.07] p-5 flex flex-col">
      <Brand />
      <Navigation />
      <div className="mt-auto glass rounded-2xl p-4">
        <div className="flex items-center gap-2 text-sm font-medium"><ShieldCheck size={16} className="text-[var(--accent)]" /> Private by default</div>
        <p className="muted text-xs leading-5 mb-0">Audio, transcripts and learning data stay on this computer.</p>
      </div>
    </aside>
    <div className="min-w-0 flex-1">
      <div className="mobile-nav border-b border-white/[.07] p-3">
        <Brand />
        <Navigation compact />
      </div>
      <main className="mx-auto max-w-[1500px] p-5 md:p-8"><Outlet /></main>
    </div>
  </div>
}

function Brand() {
  return <div className="flex items-center gap-3 px-2 py-2">
    <div className="h-9 w-9 rounded-xl bg-[var(--accent)] text-black grid place-items-center"><Sparkles size={18} /></div>
    <div><div className="font-semibold tracking-tight">English AI</div><div className="text-xs muted">Local Coach</div></div>
  </div>
}

function Navigation({ compact = false }: { compact?: boolean }) {
  return <nav className={compact ? 'mt-3 flex gap-2 overflow-x-auto pb-1' : 'mt-10 space-y-1 text-sm'} aria-label="Main navigation">
    {NAVIGATION.map(({ to, label, icon: Icon, ...options }) => <NavLink
      key={to}
      to={to}
      end={'end' in options ? options.end : false}
      className={({ isActive }) => `${compact ? 'shrink-0' : 'w-full'} flex items-center gap-2 rounded-xl px-3 py-2.5 no-underline ${isActive ? 'bg-white/[.08] text-white' : 'muted'}`}
    ><Icon size={17} /><span>{label}</span></NavLink>)}
  </nav>
}
