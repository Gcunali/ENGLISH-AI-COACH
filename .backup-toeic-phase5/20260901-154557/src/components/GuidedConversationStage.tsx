import { Mic, Square, Volume2 } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  cancelCurrentTeacherResponse,
  finishGuidedConversation,
  getGuidedLessonSession,
  startGuidedConversation,
  stopGuidedConversation,
  subscribeVoiceEngineEvents,
} from "../services/native";
import type {
  GuidedActiveContent,
  GuidedLessonSession,
  VoiceEngineEvent,
} from "../types";

type Content = Extract<GuidedActiveContent, { kind: "guided_conversation" }>;
type Status =
  | "idle"
  | "starting"
  | "listening"
  | "recording"
  | "transcribing"
  | "thinking"
  | "speaking"
  | "cancelling"
  | "error";

export function GuidedConversationStage({
  content,
  session,
  stageId,
  update,
  reportError,
}: {
  content: Content;
  session: GuidedLessonSession;
  stageId: string;
  update: (s: GuidedLessonSession) => void;
  reportError: (s: string | null) => void;
}) {
  const [status, setStatus] = useState<Status>("idle");
  const [draft, setDraft] = useState("");
  const [busy, setBusy] = useState(false);
  const mounted = useRef(true);
  const refresh = useCallback(async () => {
    const value = await getGuidedLessonSession(session.id);
    if (value && mounted.current) update(value);
  }, [session.id, update]);
  useEffect(() => {
    mounted.current = true;
    let off = () => {};
    void subscribeVoiceEngineEvents((event) => {
      onEvent(event, setStatus, setDraft, reportError);
      if (event.type === "transcript" || event.type === "teacher_response")
        void refresh();
    }).then((x) => (off = x));
    return () => {
      mounted.current = false;
      off();
      void stopGuidedConversation();
    };
  }, [refresh, reportError, session.id]);
  const active = status !== "idle" && status !== "error";
  const safe =
    status === "idle" || status === "listening" || status === "error";
  const minimum = content.studentTurnCount >= content.minimumStudentTurns;
  const maximum = content.studentTurnCount >= content.maximumStudentTurns;
  useEffect(() => {
    if (maximum && active)
      void stopGuidedConversation().then(() => setStatus("idle"));
  }, [maximum, active]);
  const start = async () => {
    setBusy(true);
    reportError(null);
    setStatus("starting");
    try {
      update(await startGuidedConversation(session.id, stageId));
    } catch (e) {
      setStatus("error");
      reportError(message(e));
    } finally {
      setBusy(false);
    }
  };
  const finish = async () => {
    setBusy(true);
    reportError(null);
    try {
      if (active) await stopGuidedConversation();
      update(await finishGuidedConversation(session.id, stageId));
      setStatus("idle");
    } catch (e) {
      reportError(message(e));
    } finally {
      setBusy(false);
    }
  };
  const targets = useMemo(
    () => [...content.targetVocabulary, ...content.targetExpressions],
    [content],
  );
  const lastStudent = content.turns.at(-1)?.role === "student";
  return (
    <div className="mt-6 grid gap-5">
      <section className="grid gap-3 rounded-2xl border border-white/10 bg-white/[.025] p-5 sm:grid-cols-2">
        <Fact label="Scenario" value={content.scenario} />
        <Fact label="Your role" value={content.studentRole} />
        <Fact label="Teacher role" value={content.teacherRole} />
        <Fact label="Goal" value={content.goal} />
        <div className="sm:col-span-2">
          <p className="eyebrow">Target language</p>
          <div className="flex flex-wrap gap-2">
            {targets.map((x) => (
              <span
                key={x}
                className="rounded-full bg-white/[.06] px-3 py-1 text-xs"
              >
                {x}
              </span>
            ))}
          </div>
          <p className="muted mb-0 mt-2 text-xs">
            Helpful language, not a required checklist.
          </p>
        </div>
      </section>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <strong>
            {content.studentTurnCount} / {content.recommendedStudentTurns}{" "}
            recommended turns
          </strong>
          <p className="muted mb-0 mt-1 text-xs">
            Minimum {content.minimumStudentTurns} · maximum{" "}
            {content.maximumStudentTurns}
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          {!active && (!maximum || lastStudent) && (
            <button
              className="button-primary"
              disabled={busy}
              onClick={() => void start()}
            >
              <Mic size={16} />
              {lastStudent ? "Retry Teacher Response" : content.started ? "Resume Conversation" : "Start Conversation"}
            </button>
          )}
          {(status === "thinking" ||
            status === "speaking" ||
            status === "cancelling") && (
            <button
              className="button-secondary"
              onClick={() => void cancelCurrentTeacherResponse()}
            >
              <Square size={15} />
              Stop Response
            </button>
          )}
          <button
            className="button-primary"
            disabled={busy || !minimum || !safe}
            aria-describedby={!minimum ? "guided-finish-reason" : undefined}
            onClick={() => void finish()}
          >
            Finish Conversation
          </button>
        </div>
      </div>
      {!minimum && (
        <p id="guided-finish-reason" className="muted m-0 text-xs">
          Complete at least {content.minimumStudentTurns} speaking turns before
          finishing.
        </p>
      )}
      {content.studentTurnCount >= content.recommendedStudentTurns && (
        <p className="m-0 text-sm text-emerald-300">
          Recommended practice reached.
        </p>
      )}
      {maximum && (
        <p className="m-0 text-sm">
          You've reached the end of this conversation practice. Finish when
          you're ready.
        </p>
      )}
      <p
        role="status"
        aria-live="polite"
        className="m-0 flex items-center gap-2 text-sm"
      >
        <Volume2 size={15} />
        {statusLabel(status)}
      </p>
      <ol
        aria-label="Conversation transcript"
        className="grid max-h-[32rem] gap-3 overflow-y-auto p-0"
      >
        {content.turns.map((turn) => (
          <li
            key={turn.id}
            className={`list-none rounded-2xl p-4 ${turn.role === "student" ? "ml-auto max-w-[85%] bg-[var(--accent)]/10" : "mr-auto max-w-[85%] bg-white/[.05]"}`}
          >
            <strong className="text-xs">
              {turn.role === "student" ? "You" : "Teacher"}
            </strong>
            <p className="mb-0 mt-1 whitespace-pre-wrap text-sm">{turn.text}</p>
            {turn.partial && (
              <span className="muted text-[10px]">Response stopped</span>
            )}
          </li>
        ))}
        {draft && (
          <li className="mr-auto max-w-[85%] list-none rounded-2xl border border-white/10 p-4">
            <strong className="text-xs">Teacher</strong>
            <p className="mb-0 mt-1 text-sm">{draft}</p>
          </li>
        )}
      </ol>
    </div>
  );
}
function Fact({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <p className="eyebrow">{label}</p>
      <p className="mb-0 text-sm">{value}</p>
    </div>
  );
}
function onEvent(
  e: VoiceEngineEvent,
  set: (s: Status) => void,
  draft: (s: string) => void,
  error: (s: string | null) => void,
) {
  if (e.type === "engine_started") set("starting");
  else if (e.type === "listening") set("listening");
  else if (e.type === "student_speaking") set("recording");
  else if (e.type === "transcribing") set("transcribing");
  else if (e.type === "teacher_thinking") set("thinking");
  else if (e.type === "teacher_response_delta") {
    set("thinking");
    draft(e.text);
  } else if (e.type === "teacher_speaking") set("speaking");
  else if (e.type === "teacher_response") {
    draft("");
  } else if (e.type === "teacher_cancel_requested") set("cancelling");
  else if (e.type === "teacher_cancelled") set("listening");
  else if (e.type === "engine_stopped") set("idle");
  else if (e.type === "error") {
    set("error");
    error(e.message);
  }
}
function statusLabel(s: Status) {
  return {
    idle: "Conversation paused.",
    starting: "Starting conversation…",
    listening: "Your turn. Listening…",
    recording: "Listening…",
    transcribing: "Transcribing locally…",
    thinking: "Teacher thinking…",
    speaking: "Teacher speaking…",
    cancelling: "Stopping response…",
    error: "Voice temporarily unavailable.",
  }[s];
}
function message(v: unknown) {
  return v instanceof Error ? v.message : String(v);
}
