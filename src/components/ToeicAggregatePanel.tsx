import { useMemo, useState } from "react";
import type { ToeicAggregatePart } from "../types";

function countReview(value: unknown): number {
  if (!Array.isArray(value)) return 0;
  return value.reduce((sum, item) => { if (!item || typeof item !== "object") return sum + 1; const row=item as {questions?:unknown;feedback?:{questions?:unknown}}; return sum + (row.questions ? countReview(row.questions) : row.feedback?.questions ? countReview(row.feedback.questions) : 1); }, 0);
}

export function ToeicAggregatePanel({ parts, label }: { parts: ToeicAggregatePart[]; label: string }) {
  const [part, setPart] = useState(parts[0]?.partNumber ?? 0);
  const selected = parts.find((item) => item.partNumber === part);
  const stats = useMemo(() => parts.map((item) => {
    const result = item.result ?? {};
    return { part: item.partNumber, correct: Number(result.correct ?? 0), total: Number(result.total ?? 0), review: countReview(item.review) };
  }), [parts]);
  return <section className="mt-4" aria-label={label}>
    <div className="toeic-parts-grid">{stats.map((item) => <button type="button" className={`toeic-part-row ${part === item.part ? "available" : ""}`} key={item.part} onClick={() => setPart(item.part)}><span><strong>Part {item.part}</strong><small>{item.correct}/{item.total} correct · {item.review} review items</small></span></button>)}</div>
    {selected && <div className="mt-3"><h3>Part {selected.partNumber}</h3><pre className="toeic-aggregate-review">{JSON.stringify(selected.review, null, 2)}</pre></div>}
  </section>;
}
