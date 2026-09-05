import {
  BookOpenCheck,
  Check,
  Clock3,
  Headphones,
  History,
  LockKeyhole,
} from "lucide-react";
import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ErrorState, LoadingState } from "../components/PageState";
import {
  AppCard,
  InlineNotice,
  PageHero,
  PageShell,
  SectionHeader,
} from "../components/ProductUI";
import {
  getToeicOverview,
  getToeicPart2Overview,
  getToeicPart3Overview,
  getToeicPart4Overview,
  getToeicPart5Overview,
  getToeicPart6Overview,
  getToeicPart7Overview,
  startToeicPart2Session,
  startToeicPart3Session,
  startToeicPart4Session,
  startToeicPart5Session,
  startToeicPart6Session,
  startToeicPart7Session,
  startToeicSession,
} from "../services/native";
import type {
  ToeicOverview,
  ToeicPart2Overview,
  ToeicPart3Overview,
  ToeicPart4Overview,
  ToeicPart5Overview,
  ToeicPart6Overview,
  ToeicPart7Overview,
} from "../types";
export function ToeicPage() {
  const nav = useNavigate();
  const [p1, setP1] = useState<ToeicOverview | null>(null);
  const [p2, setP2] = useState<ToeicPart2Overview | null>(null);
  const [p3, setP3] = useState<ToeicPart3Overview | null>(null);
  const [p4, setP4] = useState<ToeicPart4Overview | null>(null);
  const [p5, setP5] = useState<ToeicPart5Overview | null>(null);
  const [p6, setP6] = useState<ToeicPart6Overview | null>(null);
  const [p7, setP7] = useState<ToeicPart7Overview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [fullFamily, setFullFamily] = useState<"A" | "B" | "C">("A");
  const load = () => {
    setError(null);
    void Promise.all([
      getToeicOverview(),
      getToeicPart2Overview(),
      getToeicPart3Overview(),
      getToeicPart4Overview(),
      getToeicPart5Overview(),
      getToeicPart6Overview(),
      getToeicPart7Overview(),
    ])
      .then(([a, b, c, d, e, f, g]) => {
        setP1(a);
        setP2(b);
        setP3(c);
        setP4(d);
        setP5(e);
        setP6(f);
        setP7(g);
      })
      .catch((x) => setError(msg(x)));
  };
  useEffect(load, []);
  if (error && !p1) return <ErrorState message={error} onRetry={load} />;
  if (!p1 || !p2 || !p3 || !p4 || !p5 || !p6 || !p7)
    return <LoadingState label="Loading the local TOEIC item bank…" />;
  const start = async (id: string, v: number, part: number) => {
    setBusy(id);
    try {
      if (part === 7) {
        const s = await startToeicPart7Session(id, v, "learning");
        nav(`/toeic/part7/session/${s.sessionId}`);
      } else if (part === 6) {
        const s = await startToeicPart6Session(id, v, "learning");
        nav(`/toeic/part6/session/${s.sessionId}`);
      } else if (part === 5) {
        const s = await startToeicPart5Session(id, v, "learning");
        nav(`/toeic/part5/session/${s.sessionId}`);
      } else if (part === 4) {
        const s = await startToeicPart4Session(id, v);
        nav(`/toeic/part4/session/${s.sessionId}`);
      } else if (part === 3) {
        const s = await startToeicPart3Session(id, v);
        nav(`/toeic/part3/session/${s.sessionId}`);
      } else if (part === 2) {
        const s = await startToeicPart2Session(id, v);
        nav(`/toeic/part2/session/${s.sessionId}`);
      } else {
        const s = await startToeicSession(id, v);
        nav(`/toeic/session/${s.sessionId}`);
      }
    } catch (x) {
      setError(msg(x));
    } finally {
      setBusy(null);
    }
  };
  return (
    <PageShell width="wide">
      <PageHero
        eyebrow="Exam preparation"
        title="TOEIC Preparation"
        accent="Listening & Reading"
        description="Offline TOEIC-style practice with deterministic grading. Listening and Reading Parts 1–7 are available."
        compact
      />
      {error && (
        <InlineNotice tone="warning" live>
          {error}
        </InlineNotice>
      )}
      <div className="toeic-trust-row">
        <span>
          <Clock3 />
          Untimed by design
        </span>
        <span>
          <LockKeyhole />
          Answers stored locally
        </span>
        <span>
          <BookOpenCheck />
          Authored explanations
        </span>
      </div>
      <Forms
        title="Listening Part 1 — Photographs"
        description="Look at each photograph and choose the best spoken description."
        forms={p1.forms}
        total={6}
        busy={busy}
        open={(id, v, a) =>
          a ? nav(`/toeic/session/${a}`) : void start(id, v, 1)
        }
      />
      <Forms
        title="Listening Part 2 — Question–Response"
        description="Listen to a prompt and three responses, then choose the best response."
        forms={p2.forms}
        total={25}
        busy={busy}
        open={(id, v, a) =>
          a ? nav(`/toeic/part2/session/${a}`) : void start(id, v, 2)
        }
      />
      <Forms
        title="Listening Part 3 — Conversations"
        description="Read three questions, listen once, and lock all answers before feedback."
        forms={p3.forms}
        total={39}
        busy={busy}
        open={(id, v, a) =>
          a ? nav(`/toeic/part3/session/${a}`) : void start(id, v, 3)
        }
      />
      <Forms
        title="Listening Part 4 — Talks"
        description="Read three questions, listen once, and lock all answers before feedback."
        forms={p4.forms}
        total={30}
        busy={busy}
        open={(id, v, a) =>
          a ? nav(`/toeic/part4/session/${a}`) : void start(id, v, 4)
        }
      />
      <Forms
        title="Reading Part 5 — Incomplete Sentences"
        description="Choose the word or phrase that best completes each sentence."
        forms={p5.forms}
        total={30}
        busy={busy}
        open={(id, v, a) =>
          a ? nav(`/toeic/part5/session/${a}`) : void start(id, v, 5)
        }
      />
      <Forms
        title="Reading Part 6 — Text Completion"
        description="Complete four functional texts. Feedback unlocks only after all four blanks in each text."
        forms={p6.forms}
        total={16}
        busy={busy}
        open={(id, v, a) =>
          a ? nav(`/toeic/part6/session/${a}`) : void start(id, v, 6)
        }
      />
      <Forms title="Reading Part 7 — Reading Comprehension" description="Read single, double, and triple document sets. Feedback unlocks only after each complete set." forms={p7.forms} total={54} busy={busy} open={(id,v,a)=>a?nav(`/toeic/part7/session/${a}`):void start(id,v,7)}/>
      <AppCard className="mt-4">
        <div className="section-header">
          <div>
            <p className="eyebrow">Full exam architecture</p>
            <h2 className="section-title">
              Listening & Reading · 200 questions
            </h2>
            <p className="section-description">
              Listening and Reading are structurally complete: 100 questions per section, 200 total.
            </p>
          </div>
          <div className="toeic-result-actions">
            <label className="grid gap-1 text-sm font-semibold">
              Full simulation family
              <select
                aria-label="Full simulation family"
                className="rounded-xl border border-[var(--line)] bg-[var(--surface)] px-3 py-2"
                value={fullFamily}
                onChange={(event) => setFullFamily(event.target.value as "A" | "B" | "C")}
              >
                <option value="A">Form family A</option>
                <option value="B">Form family B</option>
                <option value="C">Form family C</option>
              </select>
            </label>
            <button
              className="button-primary"
              onClick={() => nav(`/toeic/listening?family=${fullFamily}`)}
            >
              <Clock3 />
              Full Listening Simulation
            </button>
            <button className="button-primary" onClick={()=>nav(`/toeic/reading?family=${fullFamily}`)}><BookOpenCheck/>Full Reading Simulation</button>
            <button className="button-secondary" onClick={()=>nav(`/toeic/full?family=${fullFamily}`)}><Check/>Full Listening & Reading</button>
            <button
              className="button-secondary"
              onClick={() => nav("/toeic/history")}
            >
              <History />
              TOEIC Performance
            </button>
          </div>
        </div>
        <div className="toeic-parts-grid">
          {p1.parts.map((part, i) => (
            <div
              className={`toeic-part-row ${part.runtimeAvailable ? "available" : ""}`}
              key={part.part}
            >
              <span>{part.runtimeAvailable ? <Check /> : <LockKeyhole />}</span>
              <div>
                <strong>
                  Part {i + 1} — {part.title}
                </strong>
                <small>{part.questionCount} questions</small>
              </div>
              <b>{part.runtimeAvailable ? "Available" : "Not yet available"}</b>
            </div>
          ))}
        </div>
      </AppCard>
      <p className="toeic-disclaimer">{p1.disclaimer}</p>
    </PageShell>
  );
}
function Forms({
  title,
  description,
  forms,
  total,
  busy,
  open,
}: {
  title: string;
  description: string;
  forms: Array<{
    formId: string;
    formVersion: number;
    title: string;
    activeSessionId: string | null;
  }>;
  total: number;
  busy: string | null;
  open: (id: string, v: number, a: string | null) => void;
}) {
  return (
    <AppCard className="mt-4">
      <SectionHeader title={title} description={description} />
      <div className="toeic-form-grid">
        {forms.map((f) => (
          <article className="toeic-form-card" key={f.formId}>
            <div className="toeic-part-icon">
              <Headphones />
            </div>
            <div>
              <p className="eyebrow">{total} questions · Untimed</p>
              <h3>{f.title}</h3>
              <p>
                Original local content with deterministic scoring and
                explanations.
              </p>
            </div>
            <button
              className="button-primary"
              disabled={busy !== null}
              onClick={() => open(f.formId, f.formVersion, f.activeSessionId)}
            >
              {f.activeSessionId
                ? "Resume"
                : busy === f.formId
                  ? "Starting…"
                  : "Start form"}
            </button>
          </article>
        ))}
      </div>
    </AppCard>
  );
}
function msg(x: unknown) {
  return x instanceof Error ? x.message : String(x);
}
