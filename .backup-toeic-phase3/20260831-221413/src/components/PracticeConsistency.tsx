import { Award, Flame, Target, Timer } from 'lucide-react'
import { Link } from 'react-router-dom'
import { useGamificationOverview } from '../hooks/useGamification'
import { ErrorState, LoadingState } from './PageState'

export function PracticeConsistency({ compact = false }: { compact?: boolean }) {
  const query = useGamificationOverview()
  if (query.loading) return <LoadingState label="Loading local practice consistency…" />
  if (query.error || !query.data) return <ErrorState message={query.error ?? 'Practice data unavailable.'} onRetry={query.reload} />
  const value = query.data
  return <section className="glass rounded-[24px] p-5 md:p-6" aria-label="Practice Consistency">
    <div className="flex flex-wrap items-start justify-between gap-3"><div><p className="muted mb-1 text-[10px] uppercase tracking-widest">Practice Consistency</p><h2 className="m-0 text-lg">Level {value.practiceLevel} · {value.totalXp} XP</h2></div><Link to="/achievements" className="rounded-full border border-white/15 px-4 py-2 text-xs text-white no-underline">View achievements</Link></div>
    <div className="mt-4 grid gap-3 sm:grid-cols-2 2xl:grid-cols-4">
      <Metric icon={Flame} label="Current streak" value={`${value.currentStreakDays} days`} />
      <Metric icon={Timer} label="Qualifying practice" value={`${value.totalPracticeMinutes} min`} />
      <Metric icon={Target} label="This week" value={`${value.weeklyGoal.practicedMinutes}/${value.weeklyGoal.goalMinutes} min`} />
      <Metric icon={Award} label="Achievements" value={`${value.unlockedAchievementCount}/${value.totalAchievementCount}`} />
    </div>
    <progress className="semantic-progress mt-4" value={value.weeklyGoal.progressPercent} max={100} aria-label={`${value.weeklyGoal.progressPercent}% of weekly goal`} />
    {!compact && <p className="muted mb-0 mt-3 text-xs">Practice Level and XP measure consistency only. They are not CEFR proficiency scores.</p>}
  </section>
}

function Metric({ icon: Icon, label, value }: { icon: typeof Flame; label: string; value: string }) { return <div className="rounded-2xl bg-white/[.035] p-4"><Icon size={16} className="text-[var(--accent)]" /><strong className="mt-2 block text-lg">{value}</strong><span className="muted text-[10px] uppercase tracking-wider">{label}</span></div> }
