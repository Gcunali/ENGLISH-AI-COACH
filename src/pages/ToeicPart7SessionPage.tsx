import { useEffect, useMemo, useState } from "react";
import {
  CheckCircle2,
  ChevronRight,
  CircleAlert,
  Pause,
  TextCursorInput,
} from "lucide-react";
import { useNavigate, useParams } from "react-router-dom";
import { ErrorState, LoadingState } from "../components/PageState";
import {
  AppCard,
  InlineNotice,
  PageHeader,
  PageShell,
} from "../components/ProductUI";
import {
  advanceToeicPart7Session,
  getToeicPart7Session,
  submitToeicPart7Answer,
} from "../services/native";
import type { ToeicPart7Session } from "../types";

const message = (x: unknown) => (x instanceof Error ? x.message : String(x));
const passage = (value: string, active?: string) =>
  value.split(/(\[\d+\])/).map((part, i) => {
    const id = part.match(/^\[(\d+)\]$/)?.[1];
    return id ? (
      <mark
        key={`${id}-${i}`}
        className={id === active ? "toeic-p6-blank active" : "toeic-p6-blank"}
        aria-label={`Blank ${id}`}
      >
        {part}
      </mark>
    ) : (
      part
    );
  });

export function ToeicPart7SessionPage() {
  const { sessionId } = useParams();
  const navigate = useNavigate();
  const query = new URLSearchParams(location.search);
  const practiceId = query.get("toeicPractice");
  const practiceLimit = Number(query.get("limit"));
  const [data, setData] = useState<ToeicPart7Session | null>(null);
  const [selected, setSelected] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [showCompleted, setShowCompleted] = useState(false);
  const load = () => {
    if (!sessionId) return;
    setError(null);
    void getToeicPart7Session(sessionId)
      .then((s) => {
        if (s.status === "completed")
          navigate(`/toeic/part7/results/${s.sessionId}`, { replace: true });
        else setData(s);
      })
      .catch((x) => setError(message(x)));
  };
  useEffect(load, [sessionId]);
  const current = useMemo(() => {
    const set = data?.currentSet;
    return set?.questions.find((q) => !q.locked) ?? null;
  }, [data]);
  const submit = async () => {
    if (!data || !current || !selected) return;
    setBusy(true);
    setError(null);
    try {
      setData(
        await submitToeicPart7Answer(
          data.sessionId,
          current.itemId,
          current.itemVersion,
          selected,
        ),
      );
      setSelected("");
    } catch (x) {
      setError(message(x));
    } finally {
      setBusy(false);
    }
  };
  const next = async () => {
    if (!data) return;
    if (practiceId && data.answeredCount >= practiceLimit) {
      navigate(`/toeic/personalized/${practiceId}`);
      return;
    }
    setBusy(true);
    try {
      const s = await advanceToeicPart7Session(data.sessionId);
      if (s.status === "completed")
        navigate(`/toeic/part7/results/${s.sessionId}`);
      else {
        setData(s);
        setShowCompleted(false);
      }
    } catch (x) {
      setError(message(x));
    } finally {
      setBusy(false);
    }
  };
  if (error && !data) return <ErrorState message={error} onRetry={load} />;
  if (!data || !data.currentSet)
    return <LoadingState label="Restoring your untimed Part 7 session…" />;
  const set = data.currentSet;
  const feedback = data.setFeedback;
  return (
    <PageShell width="wide">
      <PageHeader
        eyebrow="TOEIC Reading · Part 7"
        title={`Reading set ${set.setNumber} of 15 — ${set.title}`}
        description={`${data.answeredCount}/54 answers locked · Untimed · ${set.documentType.replaceAll("_", " ")}`}
      />
      {error && (
        <InlineNotice tone="warning" live>
          {error}
        </InlineNotice>
      )}
      <div className="toeic-p6-layout">
        <AppCard className="toeic-p6-passage">
          <p className="eyebrow">
            <TextCursorInput /> Passage
          </p>
          <div className="toeic-p6-document">
            {passage(
              showCompleted && feedback ? feedback.completedText : set.passage,
              current?.blankId,
            )}
          </div>
          {feedback && (
            <button
              className="button-secondary mt-3"
              onClick={() => setShowCompleted((v) => !v)}
            >
              {showCompleted ? "Show original blanks" : "Show completed text"}
            </button>
          )}
        </AppCard>
        <AppCard>
          {feedback ? (
            <>
              <div className="section-header">
                <div>
                  <p className="eyebrow">Passage results</p>
                  <h2 className="section-title">
                    All answers are now reviewed
                  </h2>
                </div>
                <b>{feedback.questions.filter((q) => q.isCorrect).length}/{feedback.questions.length}</b>
              </div>
              <div className="toeic-p6-feedback-list">
                {feedback.questions.map((q) => (
                  <article
                    key={q.blankId}
                    className={
                      q.isCorrect
                        ? "toeic-feedback correct"
                        : "toeic-feedback wrong"
                    }
                  >
                    <h3>
                      {q.isCorrect ? <CheckCircle2 /> : <CircleAlert />}{" "}
                      Question {q.questionNumber} —{" "}
                      {q.isCorrect ? "Correct" : "Incorrect"}
                    </h3>
                    {!q.isCorrect && (
                      <>
                        <p>
                          <strong>Your answer:</strong> {q.selectedChoice} ·{" "}
                          <strong>Correct:</strong> {q.correctChoice}
                        </p>
                        <p>
                          <strong>Completed context:</strong>{" "}
                          {q.completedContext}
                        </p>
                        <p>
                          <strong>Why {q.correctChoice} is correct:</strong>{" "}
                          {q.correctExplanation}
                        </p>
                        <p>
                          <strong>Why {q.selectedChoice} does not work:</strong>{" "}
                          {q.selectedDistractorExplanation}
                        </p>
                      </>
                    )}
                    <p>
                      <strong>Reading focus:</strong>{" "}
                      {q.skillCategory.replaceAll("_", " ")}
                    </p>
                    {q.usefulPattern && (
                      <p>
                        <strong>Useful pattern:</strong> {q.usefulPattern}
                      </p>
                    )}
                  </article>
                ))}
              </div>
              <button
                className="button-primary mt-3"
                onClick={next}
                disabled={busy}
              >
                {set.setNumber === 15
                  ? "View Part 7 results"
                  : "Continue to next text"}
                <ChevronRight />
              </button>
            </>
          ) : (
            <>
              <p className="eyebrow">
                Question {current?.questionNumber} of 54
              </p>
              <h2 className="section-title">{current?.blankId}</h2>
              <div
                className="toeic-choice-list"
                role="radiogroup"
                aria-label={`Question ${current?.questionNumber} choices`}
              >
                {current?.choices.map((c) => (
                  <button
                    key={c.choice}
                    role="radio"
                    aria-checked={selected === c.choice}
                    className={`toeic-choice ${selected === c.choice ? "selected" : ""}`}
                    onClick={() => setSelected(c.choice)}
                  >
                    <b>{c.choice}</b>
                    <span>{c.text}</span>
                  </button>
                ))}
              </div>
              <button
                className="button-primary mt-3"
                onClick={submit}
                disabled={!selected || busy}
              >
                Submit answer
              </button>
              {set.questions.some((q) => q.locked) && (
                <InlineNotice tone="info">
                  Answer recorded. Correctness stays hidden until all questions
                  are locked.
                </InlineNotice>
              )}
            </>
          )}
        </AppCard>
      </div>
      <button className="button-ghost mt-3" onClick={() => navigate(practiceId ? `/toeic/personalized/${practiceId}` : "/toeic")}>
        <Pause />
        Pause & exit — your answers are saved
      </button>
    </PageShell>
  );
}
