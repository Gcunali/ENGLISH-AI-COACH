import { BookOpenCheck, ChevronRight, Pause } from "lucide-react";
import { useEffect, useState } from "react";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import { ErrorState, LoadingState } from "../components/PageState";
import { AppCard, InlineNotice, PageHeader, PageShell } from "../components/ProductUI";
import { ToeicAggregatePanel } from "../components/ToeicAggregatePanel";
import { getToeicFullReading, getToeicFullReadingAggregate, startToeicFullReading } from "../services/native";
import type { ToeicAggregatePart, ToeicFullReadingSession } from "../types";

export function ToeicFullReadingPage() {
  const { id } = useParams(); const nav = useNavigate(); const [search] = useSearchParams();
  const requested = search.get("family")?.toUpperCase();
  const family: "A" | "B" | "C" = requested === "B" || requested === "C" ? requested : "A";
  const [data, setData] = useState<ToeicFullReadingSession | null>(null);
  const [aggregate, setAggregate] = useState<ToeicAggregatePart[] | null>(null);
  const [mistakes, setMistakes] = useState(true); const [error, setError] = useState<string | null>(null);
  const load = () => { setError(null); void (id ? getToeicFullReading(id) : startToeicFullReading("simulation", family)).then((s) => { setData(s); if (!id) nav(`/toeic/reading/${s.sessionId}`, { replace: true }); }).catch((x) => setError(x instanceof Error ? x.message : String(x))); };
  useEffect(load, [id, family]);
  const loadReview = (only: boolean) => { if (!data) return; setMistakes(only); void getToeicFullReadingAggregate(data.sessionId, only).then(setAggregate).catch((x) => setError(String(x))); };
  if (error) return <ErrorState message={error} onRetry={load} />; if (!data) return <LoadingState label="Restoring Full Reading…" />;
  const current = data.parts.find((p) => p.status !== "completed");
  return <PageShell width="wide"><PageHeader eyebrow={`Untimed TOEIC-style Reading Simulation · Family ${data.family}`} title={data.status === "completed" ? "Reading complete" : "Full Reading · 100 questions"} description={`${data.answeredCount}/100 first answers locked · Simulation Mode · Pause anytime`} /><InlineNotice tone="info">No correctness, answer keys, completed passages or evidence are shown until all 100 questions are complete.</InlineNotice>
    {data.status === "completed" && data.estimate ? <AppCard><h2>Estimated Reading: {data.estimate.estimatedScore}</h2><p>Raw {data.estimate.rawCorrect}/100 · Range {data.estimate.rangeLow}–{data.estimate.rangeHigh} · Profile v{data.estimate.profileVersion}</p><p>{data.disclaimer}</p><div className="button-row"><button className="button-primary" onClick={() => loadReview(true)}>Review Mistakes</button><button className="button-secondary" onClick={() => loadReview(false)}>Review All</button></div>{aggregate && <ToeicAggregatePanel key={String(mistakes)} parts={aggregate} label={mistakes ? "Reading mistakes" : "All Reading answers"} />}</AppCard> : <AppCard><h2>{current ? `Part ${current.partNumber} — ${current.title}` : "Preparing result"}</h2><div className="toeic-parts-grid">{data.parts.map((p) => <div className={`toeic-part-row ${p.status === "completed" ? "available" : ""}`} key={p.partNumber}><BookOpenCheck /><div><strong>Part {p.partNumber} — {p.title}</strong><small>{p.questionCount} questions · {p.status}</small></div></div>)}</div>{current && <button className="button-primary mt-3" onClick={() => nav(current.route)}>Continue Part {current.partNumber}<ChevronRight /></button>}</AppCard>}
    <button className="button-ghost mt-3" onClick={() => nav("/toeic")}><Pause />Pause & exit</button></PageShell>;
}
