import { Headphones, Play } from "lucide-react";
import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ErrorState, LoadingState } from "../components/PageState";
import { AppCard, PageHeader, PageShell } from "../components/ProductUI";
import { listToeicFullLrHistory, listToeicFullReadingHistory, listToeicHistory } from "../services/native";
import type { ToeicFullLrHistory, ToeicFullReadingHistory, ToeicHistoryEntry } from "../types";

export function ToeicHistoryPage() {
  const navigate = useNavigate();
  const [data, setData] = useState<ToeicHistoryEntry[] | null>(null);
  const [reading, setReading] = useState<ToeicFullReadingHistory[]>([]);
  const [full, setFull] = useState<ToeicFullLrHistory[]>([]);
  const [error, setError] = useState<string | null>(null);
  const load = () => {
    setError(null);
    void Promise.all([listToeicHistory(), listToeicFullReadingHistory(), listToeicFullLrHistory()])
      .then(([parts, readingRows, fullRows]) => { setData(parts); setReading(readingRows); setFull(fullRows); })
      .catch((value) =>
        setError(value instanceof Error ? value.message : String(value)),
      );
  };
  useEffect(load, []);
  if (error) return <ErrorState message={error} onRetry={load} />;
  if (!data) return <LoadingState label="Loading TOEIC Performance…" />;
  return (
    <PageShell width="standard">
      <PageHeader
        eyebrow="Separate exam history"
        title="TOEIC Performance"
        description="Listening Parts 1–4 and Reading Parts 5–7. These results do not affect Course, CEFR, XP, streak, or learning memory."
      />
      <AppCard>
        <h2>Full TOEIC Listening & Reading</h2>
        <div className="toeic-history-list">{full.map((entry) => <button className="toeic-history-row" key={entry.sessionId} onClick={() => navigate(`/toeic/full/${entry.sessionId}`)}><span className="toeic-part-icon"><Headphones /></span><span><strong>Full TOEIC · Form {entry.family}</strong><small>{new Date(entry.createdAt).toLocaleString()} · {entry.status}</small></span><span><b>{entry.totalRaw ?? 0}/200</b><small>{entry.totalEstimate == null ? "Resume" : `Estimated ${entry.totalEstimate}/990`}</small></span><Play /></button>)}</div>
        <h2>Full Reading</h2>
        <div className="toeic-history-list">{reading.map((entry) => <button className="toeic-history-row" key={entry.sessionId} onClick={() => navigate(`/toeic/reading/${entry.sessionId}`)}><span className="toeic-part-icon"><Headphones /></span><span><strong>Full Reading · Form {entry.family}</strong><small>{new Date(entry.createdAt).toLocaleString()} · {entry.status} · Part {entry.currentPart}</small></span><span><b>{entry.rawCorrect ?? entry.answeredCount}/100</b><small>{entry.estimatedScore == null ? `${entry.answeredCount}/100 answered` : `Estimated ${entry.estimatedScore}/495 · v${entry.scoreProfileVersion}`}</small></span><Play /></button>)}</div>
        <h2>Part Practice</h2>
        {data.length === 0 && reading.length === 0 && full.length === 0 ? (
          <div className="empty-state">
            <Headphones />
            <h2>No TOEIC attempts yet</h2>
            <button
              className="button-primary"
              onClick={() => navigate("/toeic")}
            >
              Choose a form
            </button>
          </div>
        ) : (
          <div className="toeic-history-list">
            {data.map((entry) => {
              const part = entry.formId.startsWith("toeic-part7-")
                ? 7
                : entry.formId.startsWith("toeic-part6-")
                  ? 6
                : entry.formId.startsWith("toeic-part5-")
                  ? 5
                  : entry.formId.startsWith("toeic-part4-")
                    ? 4
                    : entry.formId.startsWith("toeic-part3-")
                      ? 3
                      : entry.formId.startsWith("toeic-part2-")
                        ? 2
                        : 1;
              const base = part === 1 ? "/toeic" : `/toeic/part${part}`;
              const route = `${base}/${entry.status === "completed" ? "results" : "session"}/${entry.sessionId}`;
              return (
                <button
                  className="toeic-history-row"
                  key={entry.sessionId}
                  onClick={() => navigate(route)}
                >
                  <span className="toeic-part-icon">
                    <Headphones />
                  </span>
                  <span>
                    <strong>{entry.formTitle}</strong>
                    <small>
                      {new Date(entry.createdAt).toLocaleString()} ·{" "}
                      {entry.status.replace("_", " ")}
                    </small>
                  </span>
                  <span>
                    <b>
                      {entry.correct}/{entry.total}
                    </b>
                    <small>
                      {entry.status === "completed"
                        ? `${entry.accuracy}% accuracy`
                        : `${entry.answered}/${entry.total} answered`}
                    </small>
                  </span>
                  <Play />
                </button>
              );
            })}
          </div>
        )}
      </AppCard>
    </PageShell>
  );
}
