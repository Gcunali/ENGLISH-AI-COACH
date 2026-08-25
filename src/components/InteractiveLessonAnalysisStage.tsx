import { CheckCircle2, RotateCcw, Sparkles } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  analyzeGuidedLesson,
  finalizeGuidedLessonAnalysis,
  getGuidedLessonAnalysis,
  getGuidedLessonSession,
  retryGuidedLessonConversationAnalysis,
} from "../services/native";
import type {
  GuidedConversationScores,
  GuidedLessonAnalysis,
  GuidedLessonSession,
} from "../types";
import { InlineNotice } from "./ProductUI";

export function InteractiveLessonAnalysisStage({
  session,
  stageId,
  updateSession,
}: {
  session: GuidedLessonSession;
  stageId: string;
  updateSession: (session: GuidedLessonSession) => void;
}) {
  const [analysis, setAnalysis] = useState<GuidedLessonAnalysis | null>();
  const [busy, setBusy] = useState<"analyze" | "retry" | "finish" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const requestActive = useRef(false);
  const load = useCallback(async () => {
    try {
      setAnalysis(await getGuidedLessonAnalysis(session.id));
    } catch (value) {
      setError(message(value));
      setAnalysis(null);
    }
  }, [session.id]);
  useEffect(() => {
    void load();
  }, [load]);

  const run = async (kind: "analyze" | "retry") => {
    if (requestActive.current) return;
    requestActive.current = true;
    setBusy(kind);
    setError(null);
    try {
      setAnalysis(
        kind === "analyze"
          ? await analyzeGuidedLesson(session.id, stageId)
          : await retryGuidedLessonConversationAnalysis(session.id, stageId),
      );
    } catch (value) {
      setError(message(value));
      await load();
    } finally {
      requestActive.current = false;
      setBusy(null);
    }
  };

  const finish = async () => {
    if (requestActive.current) return;
    requestActive.current = true;
    setBusy("finish");
    setError(null);
    try {
      const finalized = await finalizeGuidedLessonAnalysis(session.id, stageId);
      setAnalysis(finalized);
      const updated = await getGuidedLessonSession(session.id);
      if (updated) updateSession(updated);
    } catch (value) {
      setError(message(value));
    } finally {
      requestActive.current = false;
      setBusy(null);
    }
  };

  if (analysis === undefined)
    return <p aria-live="polite" className="muted mt-6">Loading your saved lesson review…</p>;

  if (!analysis)
    return (
      <div className="mt-6 rounded-2xl border border-white/10 bg-white/[.025] p-5">
        <div className="flex items-start gap-3">
          <Sparkles className="mt-0.5 shrink-0 text-[var(--accent)]" size={19} />
          <div>
            <h3 className="m-0 text-lg">Review this lesson</h3>
            <p className="muted mb-0 mt-2 text-sm">
              See participation, exercise, pronunciation and guided-conversation feedback as separate results.
            </p>
          </div>
        </div>
        <button
          type="button"
          className="button-primary mt-5"
          disabled={busy !== null}
          onClick={() => void run("analyze")}
        >
          {busy === "analyze" ? "Reviewing your lesson…" : "Analyze Lesson"}
        </button>
        {busy === "analyze" && (
          <p className="muted mb-0 mt-3 text-sm" aria-live="polite">
            Preparing practice results and reviewing the conversation…
          </p>
        )}
        {error && <p role="alert" className="mt-3 text-sm text-red-200">{error}</p>}
      </div>
    );

  return (
    <div className="mt-6 space-y-5" aria-live="polite">
      <AnalysisResult analysis={analysis} />
      {analysis.status === "partial" && !analysis.finalizedAt && (
        <div className="flex flex-wrap gap-3">
          <button
            type="button"
            className="button-secondary"
            disabled={busy !== null}
            onClick={() => void run("retry")}
          >
            <RotateCcw size={16} />
            {busy === "retry" ? "Retrying…" : "Retry Conversation Feedback"}
          </button>
        </div>
      )}
      {!analysis.finalizedAt &&
        (analysis.status === "completed" || analysis.status === "partial") && (
          <button
            type="button"
            className="button-primary"
            disabled={busy !== null}
            onClick={() => void finish()}
          >
            <CheckCircle2 size={16} />
            {busy === "finish" ? "Completing…" : "Finish Guided Lesson"}
          </button>
        )}
      {analysis.finalizedAt && (
        <p className="muted mb-0 text-sm">This saved lesson review is final.</p>
      )}
      {error && <p role="alert" className="text-sm text-red-200">{error}</p>}
    </div>
  );
}

function AnalysisResult({ analysis }: { analysis: GuidedLessonAnalysis }) {
  const result = analysis.result;
  return (
    <>
      <header>
        <p className="eyebrow">Lesson Review</p>
        <h3 className="section-title text-xl">Your practice results</h3>
        {analysis.status === "partial" && (
          <InlineNotice tone="warning">
            Conversation feedback is unavailable, but the rest of your lesson results are ready.
          </InlineNotice>
        )}
      </header>

      <section aria-labelledby="guided-participation" className="rounded-2xl border border-white/10 p-5">
        <h4 id="guided-participation" className="m-0 text-base">Lesson participation</h4>
        <p className="muted mt-2 text-sm">
          {result.participation.completedRequiredStageCount} of {result.participation.requiredStageCount} required practice stages completed.
        </p>
        <ul className="mt-3 grid gap-2 p-0 sm:grid-cols-2">
          {result.participation.stageStatus.map((stage) => (
            <li key={stage.stageId} className="flex items-center gap-2 rounded-xl bg-white/[.035] px-3 py-2 text-sm">
              <CheckCircle2 size={14} className="text-emerald-300" />
              <span className="min-w-0 flex-1 truncate">{stage.stageType.replaceAll("_", " ")}</span>
              <span className="muted capitalize">{stage.status}</span>
            </li>
          ))}
        </ul>
        {result.participation.vocabularyItemCount > 0 && (
          <p className="muted mb-0 mt-3 text-sm">
            {result.participation.vocabularyItemCount} vocabulary items practiced.
          </p>
        )}
        {result.participation.listening && (
          <p className="muted mb-0 mt-1 text-sm">
            {result.participation.listening.listenedSegmentCount} of {result.participation.listening.segmentCount} listening segments practiced · {result.participation.listening.totalPlaybackCount} complete plays.
          </p>
        )}
      </section>

      <ConversationSection analysis={analysis} />
      <ExerciseSection analysis={analysis} />
      <PronunciationSection analysis={analysis} />

      <section aria-labelledby="guided-objectives" className="rounded-2xl border border-white/10 p-5">
        <h4 id="guided-objectives" className="m-0 text-base">Objectives practiced</h4>
        {result.practicedObjectives.length ? (
          <ul className="mb-0 mt-3 space-y-2 pl-5 text-sm">
            {result.practicedObjectives.map((objective) => <li key={objective}>{objective}</li>)}
          </ul>
        ) : (
          <p className="muted mb-0 mt-2 text-sm">No objectives were listed for this lesson.</p>
        )}
      </section>
    </>
  );
}

function ConversationSection({ analysis }: { analysis: GuidedLessonAnalysis }) {
  const conversation = analysis.result.conversation;
  return (
    <section aria-labelledby="guided-conversation-result" className="rounded-2xl border border-white/10 p-5">
      <h4 id="guided-conversation-result" className="m-0 text-base">Conversation performance</h4>
      {conversation.status === "completed" && conversation.scores ? (
        <>
          <ScoreGrid scores={conversation.scores} />
          <p className="muted text-xs">
            Conversation scores summarize this lesson's guided conversation. They are not a CEFR assessment.
          </p>
          {conversation.goalProgress && (
            <p className="text-sm"><strong>Goal progress:</strong> {goalLabel(conversation.goalProgress)}</p>
          )}
          {conversation.summary && <p className="muted text-sm">{conversation.summary}</p>}
          <ObservationList title="Strengths" items={conversation.strengths.map((item) => item.text)} />
          <ObservationList title="Focus next" items={conversation.focusAreas.map((item) => item.text)} />
        </>
      ) : conversation.status === "insufficient_evidence" ? (
        <p className="muted mb-0 mt-2 text-sm">Not enough conversation evidence for detailed scoring.</p>
      ) : conversation.status === "unavailable" || conversation.status === "pending" ? (
        <p className="muted mb-0 mt-2 text-sm">This part of your feedback is temporarily unavailable.</p>
      ) : (
        <p className="muted mb-0 mt-2 text-sm">Guided conversation was not part of this lesson.</p>
      )}
    </section>
  );
}

function ScoreGrid({ scores }: { scores: GuidedConversationScores }) {
  const values = [
    ["Grammar", scores.grammar],
    ["Vocabulary", scores.vocabulary],
    ["Conversational Fluency", scores.fluency],
    ["Interaction", scores.interaction],
  ] as const;
  return (
    <dl className="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
      {values.map(([label, score]) => (
        <div key={label} className="min-w-0 rounded-2xl bg-white/[.04] p-4">
          <dt className="muted break-words text-xs">{label}</dt>
          <dd className="m-0 mt-2 text-3xl font-semibold" aria-label={`${label}: ${score} out of 100`}>
            {score}<span className="muted text-sm">/100</span>
          </dd>
        </div>
      ))}
    </dl>
  );
}

function ExerciseSection({ analysis }: { analysis: GuidedLessonAnalysis }) {
  const exercise = analysis.result.exercises;
  return (
    <section aria-labelledby="guided-exercise-result" className="rounded-2xl border border-white/10 p-5">
      <h4 id="guided-exercise-result" className="m-0 text-base">Exercise performance</h4>
      {exercise.status === "completed" ? (
        <div className="mt-3 grid gap-2 sm:grid-cols-3">
          <ResultStat label="Selected results" value={`${exercise.selectedCorrectCount} of ${exercise.exerciseCount} correct`} />
          <ResultStat label="Exercise Accuracy" value={`${exercise.accuracyPercent}%`} />
          <ResultStat label="Total attempts" value={String(exercise.totalAttemptCount)} />
        </div>
      ) : (
        <p className="muted mb-0 mt-2 text-sm">Exercises were not part of this lesson.</p>
      )}
    </section>
  );
}

function PronunciationSection({ analysis }: { analysis: GuidedLessonAnalysis }) {
  const pronunciation = analysis.result.pronunciation;
  return (
    <section aria-labelledby="guided-pronunciation-result" className="rounded-2xl border border-white/10 p-5">
      <h4 id="guided-pronunciation-result" className="m-0 text-base">Pronunciation practice</h4>
      {pronunciation.status === "completed" ? (
        <>
          <div className="mt-3 grid gap-2 sm:grid-cols-3">
            <ResultStat label="Phrases practiced" value={String(pronunciation.selectedPhraseCount)} />
            <ResultStat label="Acoustic Match Average" value={String(pronunciation.meanAcousticMatch)} />
            <ResultStat label="Total attempts" value={String(pronunciation.totalAttemptCount)} />
          </div>
          {pronunciation.issueSummary.length > 0 && (
            <div className="mt-4">
              <h5 className="m-0 text-sm">Focus sounds</h5>
              <div className="mt-2 flex flex-wrap gap-2">
                {pronunciation.issueSummary.map((issue) => (
                  <span key={issue.phone} className="rounded-full border border-white/10 px-3 py-1 text-sm" title={issue.hint ?? undefined}>
                    /{issue.phone}/ · {issue.meanScore}
                  </span>
                ))}
              </div>
            </div>
          )}
          <p className="muted mb-0 mt-4 text-xs">
            Acoustic Match is an estimate from this microphone recording. Accent, room noise and device quality can affect it.
          </p>
        </>
      ) : (
        <p className="muted mb-0 mt-2 text-sm">Pronunciation was not practiced in this lesson.</p>
      )}
    </section>
  );
}

function ResultStat({ label, value }: { label: string; value: string }) {
  return <div className="rounded-xl bg-white/[.04] p-3"><div className="muted text-xs">{label}</div><strong className="mt-1 block break-words">{value}</strong></div>;
}

function ObservationList({ title, items }: { title: string; items: string[] }) {
  if (!items.length) return null;
  return <div className="mt-4"><h5 className="m-0 text-sm">{title}</h5><ul className="mb-0 mt-2 space-y-2 pl-5 text-sm">{items.map((item) => <li key={item} className="break-words">{item}</li>)}</ul></div>;
}

function goalLabel(value: "limited" | "partial" | "strong") {
  if (value === "strong") return "Strong progress";
  if (value === "partial") return "Partial progress";
  return "Limited evidence or progress";
}

function message(value: unknown) {
  return value instanceof Error ? value.message : String(value);
}
