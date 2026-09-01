import { Award, Check, Lock } from 'lucide-react'
import { useEffect, useState } from 'react'
import { PracticeConsistency } from '../components/PracticeConsistency'
import { ErrorState, LoadingState } from '../components/PageState'
import { useAchievements } from '../hooks/useGamification'
import { getGamificationProfile, updateWeeklyPracticeGoal } from '../services/native'
import { notifyGamificationDataChanged } from '../utils/gamificationData'
import { formatLocalDate } from '../utils/format'
import { PageHeader, PageShell } from '../components/ProductUI'

export function AchievementsPage() {
  const query = useAchievements()
  const [goal, setGoal] = useState(90)
  const [saving, setSaving] = useState(false)
  const [message, setMessage] = useState<string | null>(null)
  const [saveFailed, setSaveFailed] = useState(false)
  useEffect(() => { void getGamificationProfile().then((profile) => setGoal(profile.weeklyGoalMinutes)).catch(() => undefined) }, [])
  const saveGoal = async () => {
    setSaving(true); setMessage(null); setSaveFailed(false)
    try { const profile = await updateWeeklyPracticeGoal(goal); setGoal(profile.weeklyGoalMinutes); setMessage('Weekly goal saved locally.'); notifyGamificationDataChanged() }
    catch (reason) { setSaveFailed(true); setMessage(reason instanceof Error ? reason.message : String(reason)) }
    finally { setSaving(false) }
  }
  return <PageShell width="wide">
    <PageHeader eyebrow="Local milestones" title="Achievements" description="A transparent record of practice consistency—not a language proficiency rating." />
    <PracticeConsistency />
    <section className="glass mt-5 rounded-[24px] p-5" aria-label="Weekly practice goal"><div className="flex flex-wrap items-end gap-3"><label className="text-sm"><span className="muted mb-2 block text-xs">Weekly goal (minutes)</span><input aria-label="Weekly goal minutes" type="number" min={30} max={600} step={15} value={goal} onChange={(event) => setGoal(Number(event.target.value))} className="w-32 rounded-xl border border-white/10 bg-[#0d1119] px-3 py-2 text-white" /></label><button aria-busy={saving} disabled={saving} onClick={() => void saveGoal()} className="button-primary">{saving?'Saving…':'Save Goal'}</button>{message && <span role={saveFailed?'alert':'status'} className={saveFailed?'text-xs text-red-200':'text-xs text-[var(--success)]'}>{message}</span>}</div><p className="muted mb-0 mt-3 text-xs">Allowed range: 30–600 minutes, in 15-minute increments. The week runs Monday through Sunday.</p></section>
    {query.loading && <div className="mt-5"><LoadingState label="Loading achievements…" /></div>}
    {query.error && <div className="mt-5"><ErrorState message={query.error} onRetry={query.reload} /></div>}
    {query.data && <section className="mt-5 grid gap-4 md:grid-cols-2 xl:grid-cols-3" aria-label="Achievement list">{query.data.map((achievement) => <article key={achievement.id} className={`glass rounded-[22px] p-5 ${achievement.unlocked ? '' : 'opacity-70'}`}><div className="flex items-start justify-between gap-3"><div className={`grid h-10 w-10 place-items-center rounded-xl ${achievement.unlocked ? 'bg-[var(--accent)] text-black' : 'bg-white/[.06]'}`}>{achievement.unlocked ? <Check aria-hidden="true" size={19} /> : <Lock aria-hidden="true" size={17} />}</div><span className="muted text-[10px] uppercase tracking-wider">{achievement.category}</span></div><h2 className="mb-1 mt-4 text-base">{achievement.title}</h2><p className="muted min-h-10 text-xs leading-5">{achievement.description}</p><progress className="semantic-progress mt-4" value={achievement.progressCurrent} max={achievement.progressTarget} aria-label={`${achievement.title}: ${achievement.progressCurrent} of ${achievement.progressTarget}`}/><div className="muted mt-2 flex justify-between text-[10px]"><span>{achievement.progressCurrent}/{achievement.progressTarget}</span><span>{achievement.unlockedAt ? `Unlocked ${formatLocalDate(achievement.unlockedAt)}` : 'Locked'}</span></div></article>)}</section>}
    <p className="muted mt-5 flex items-center gap-2 text-xs"><Award size={14} /> Historical achievements are restored silently from qualifying local records.</p>
  </PageShell>
}
