import { BarChart3, BookOpenCheck, Brain, History, Play, RefreshCw, Target } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { ErrorState, LoadingState } from "../components/PageState";
import { AppCard, InlineNotice, MetricCard, PageHero, PageShell, SectionHeader, StatusBadge } from "../components/ProductUI";
import { getToeicPersonalizedDashboard, getToeicPersonalizedPractice, setToeicTargetScore, startToeicPersonalizedPractice } from "../services/native";
import type { ToeicPersonalizedDashboard, ToeicPersonalizedSession } from "../types";

const presets = [550, 650, 750, 850, 900];

export function ToeicPersonalizedPage() {
  const { id } = useParams();
  const nav = useNavigate();
  const [dashboard, setDashboard] = useState<ToeicPersonalizedDashboard | null>(null);
  const [session, setSession] = useState<ToeicPersonalizedSession | null>(null);
  const [size, setSize] = useState<10 | 15 | 20>(15);
  const [customTarget, setCustomTarget] = useState("750");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const load = useCallback(async () => {
    setError(null);
    try {
      const data = await getToeicPersonalizedDashboard();
      setDashboard(data);
      setCustomTarget(String(data.targetScore));
      setSession(id ? await getToeicPersonalizedPractice(id) : data.activePractice);
    } catch (x) { setError(message(x)); }
  }, [id]);
  useEffect(() => { void load(); }, [load]);
  const saveTarget = async (target: number) => {
    setBusy(true); setError(null);
    try { const data = await setToeicTargetScore(target); setDashboard(data); setCustomTarget(String(target)); }
    catch (x) { setError(message(x)); } finally { setBusy(false); }
  };
  const start = async (kind: "smart" | "recent_mistakes" | "daily") => {
    setBusy(true); setError(null);
    try { const value = await startToeicPersonalizedPractice(kind, kind === "daily" ? undefined : size); setSession(value); nav(`/toeic/personalized/${value.sessionId}`); }
    catch (x) { setError(message(x)); } finally { setBusy(false); }
  };
  if (error && !dashboard) return <ErrorState message={error} onRetry={() => void load()} />;
  if (!dashboard) return <LoadingState label="Building your local TOEIC preparation profile…" />;
  return <PageShell width="wide">
    <PageHero eyebrow="TOEIC personalized preparation" title="Practice for your" accent="target score" description="A deterministic, offline plan built only from your scored TOEIC attempts and the validated local question bank." compact />
    {error && <InlineNotice tone="error" live>{error}</InlineNotice>}
    <div className="toeic-personalized-metrics">
      <MetricCard label="Target score" value={dashboard.targetScore} detail="Your goal — scoring is never altered" />
      <MetricCard label="Latest estimate" value={dashboard.latestTotalEstimate ?? "No full simulation"} detail={dashboard.latestRangeLow != null ? `Unofficial range ${dashboard.latestRangeLow}–${dashboard.latestRangeHigh}` : "Complete Full L&R to establish an estimate"} />
      <MetricCard label="Listening / Reading" value={`${dashboard.latestListeningEstimate ?? "—"} / ${dashboard.latestReadingEstimate ?? "—"}`} detail="Latest valid completed simulation" />
      <MetricCard label="Estimated gap" value={dashboard.estimatedGap == null ? "—" : dashboard.estimatedGap > 0 ? `${dashboard.estimatedGap} points` : "Target reached"} detail="Planning indicator, not an exact prediction" />
      <MetricCard label="Question exposure" value={`${dashboard.exposure.uniqueSeen}/${dashboard.exposure.bankItems}`} detail={`${dashboard.exposure.unseen} unseen · ${dashboard.exposure.totalAnswers} scored answers`} />
    </div>

    <AppCard className="mt-4">
      <SectionHeader title="Set your TOEIC target" description="Choose a preset or enter any score from 10 to 990. This goal guides recommendations only." />
      <div className="toeic-target-row">
        {presets.map((value) => <button key={value} className={dashboard.targetScore === value ? "button-primary" : "button-secondary"} disabled={busy} onClick={() => void saveTarget(value)}>{value === 900 ? "900+" : value}</button>)}
        <label><span>Custom target</span><input aria-label="Custom TOEIC target score" inputMode="numeric" min={10} max={990} value={customTarget} onChange={(e) => setCustomTarget(e.target.value)} /></label>
        <button className="button-secondary" disabled={busy} onClick={() => void saveTarget(Number(customTarget))}>Save target</button>
      </div>
    </AppCard>

    {session && <AppCard className="mt-4 toeic-active-plan">
      <SectionHeader title={session.status === "completed" ? "Practice complete" : "Practice in progress"} description={`${labelKind(session.kind)} · ${session.answeredCount}/${session.requestedCount} answered`} actions={<button className="button-secondary" onClick={() => void load()}><RefreshCw /> Refresh progress</button>} />
      {session.status === "completed" ? <InlineNotice tone="success">Raw result: {session.correctCount}/{session.requestedCount} ({session.accuracy}%). Personalized practice does not produce a scaled TOEIC estimate.</InlineNotice> : <InlineNotice tone="info">Complete only the indicated quota in each step, then return here. The question snapshot is frozen for reliable resume.</InlineNotice>}
      <div className="toeic-step-grid">{session.steps.map((step) => <article key={step.stepNumber} className="toeic-form-card"><p className="eyebrow">Step {step.stepNumber} · Part {step.partNumber}</p><h3>{step.answered}/{step.quota} answered</h3><StatusBadge tone={step.status === "completed" ? "success" : step.status === "in_progress" ? "info" : "neutral"}>{step.status.replace("_", " ")}</StatusBadge><button className="button-primary" disabled={step.status === "pending" || step.status === "completed"} onClick={() => nav(step.route)}><Play /> Continue questions</button></article>)}</div>
    </AppCard>}

    {!session && <AppCard className="mt-4">
      <SectionHeader title="Recommended next sessions" description="Short sessions selected by weakness, recent mistakes and question freshness." />
      <div className="toeic-session-size"><span>Session size</span>{([10,15,20] as const).map((value) => <button key={value} className={size === value ? "button-primary" : "button-secondary"} onClick={() => setSize(value)}>{value}</button>)}</div>
      <div className="toeic-practice-actions">
        <button disabled={busy} onClick={() => void start("smart")}><Brain /><strong>Practice My Weak Areas</strong><span>Priorities + fresh questions · {size} items</span></button>
        <button disabled={busy} onClick={() => void start("recent_mistakes")}><History /><strong>Practice Recent Mistakes</strong><span>Same weakness, fresh question when possible</span></button>
        <button disabled={busy} onClick={() => void start("daily")}><BookOpenCheck /><strong>TOEIC Daily Practice</strong><span>Balanced Listening & Reading · 12 items</span></button>
      </div>
    </AppCard>}

    <div className="toeic-personalized-columns mt-4">
      <AppCard><SectionHeader title="Top priorities" description="Only sufficiently supported weaknesses become priorities." />{dashboard.priorities.length ? dashboard.priorities.map((item) => <div className="toeic-priority" key={`${item.partNumber}-${item.skill}`}><b>#{item.rank}</b><span><strong>Part {item.partNumber} · {item.skill}</strong><small>{item.reason}</small></span></div>) : <InlineNotice tone="info">More scored attempts are needed before a strong priority can be assigned.</InlineNotice>}</AppCard>
      <AppCard><SectionHeader title="Weakness profile" description="Recent first attempts receive more weight; at least five observations are required." />{dashboard.weaknesses.slice(0,8).map((item) => <div className="toeic-weakness" key={`${item.partNumber}-${item.skill}`}><span><strong>Part {item.partNumber} · {item.skill}</strong><small>{item.correct}/{item.total} first attempts</small></span><StatusBadge tone={item.label === "Priority" ? "error" : item.label === "Needs Practice" ? "warning" : item.label === "Strong" ? "success" : "neutral"}>{item.label}</StatusBadge></div>)}</AppCard>
    </div>
    <div className="toeic-personalized-columns mt-4">
      <AppCard><SectionHeader title="Recommended next" description="Deterministic recommendations from your current evidence." />{dashboard.recommendations.map((item) => <button className="toeic-history-row" key={item.title} onClick={() => nav(item.route)}><Play /><span><strong>{item.title}</strong><small>{item.description}</small></span></button>)}</AppCard>
      <AppCard><SectionHeader title="Recent personalized practice" description="Separate from your course and general review history." />{dashboard.recentPractice.length ? dashboard.recentPractice.map((item) => <div className="toeic-weakness" key={item.sessionId}><span><strong>{labelKind(item.kind)}</strong><small>{new Date(item.completedAt ?? item.createdAt).toLocaleDateString()} · raw result</small></span><b>{item.correctCount}/{item.requestedCount}</b></div>) : <InlineNotice tone="info">No completed personalized TOEIC practice yet.</InlineNotice>}</AppCard>
    </div>
    <AppCard className="mt-4"><SectionHeader title="Full simulation trend" description="Only valid, completed 200-question simulations appear here." actions={<BarChart3 />} />{dashboard.trends.length ? <div className="toeic-trend-list">{dashboard.trends.slice(-6).reverse().map((point) => <div key={point.sessionId}><strong>{point.totalRaw}/200 raw</strong><span>Unofficial estimate {point.totalEstimate ?? "—"} · Form {point.family}</span><small>{new Date(point.completedAt).toLocaleDateString()}</small></div>)}</div> : <InlineNotice tone="info">No completed Full Listening & Reading simulation yet.</InlineNotice>}</AppCard>
    <p className="toeic-disclaimer"><Target /> TOEIC is a registered trademark of ETS. This independent offline practice app is not endorsed by ETS. All score estimates are unofficial practice indicators.</p>
  </PageShell>;
}

function labelKind(kind: string) { return kind === "smart" ? "Smart Practice" : kind === "recent_mistakes" ? "Recent Mistakes" : "TOEIC Daily Practice"; }
function message(x: unknown) { return x instanceof Error ? x.message : String(x); }
