import { ArrowLeft, Check, Mic, Play, Square, X } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { ConfirmationDialog } from "../components/ConfirmationDialog";
import { GuidedExerciseStage } from "../components/GuidedExerciseStage";
import { GuidedConversationStage } from "../components/GuidedConversationStage";
import { ErrorState, LoadingState } from "../components/PageState";
import { InlineNotice, PageHeader, PageShell } from "../components/ProductUI";
import { useGuidedLessonRecorder } from "../hooks/useGuidedLessonRecorder";
import {
  abandonGuidedLesson,
  cancelGuidedLessonAudio,
  completeGuidedLessonAudio,
  completeGuidedLessonStage,
  getGuidedLessonSession,
  prepareGuidedLessonAudio,
  selectGuidedLessonPronunciationAttempt,
  skipGuidedLessonStage,
  submitGuidedLessonPronunciation,
} from "../services/native";
import type {
  GuidedActiveContent,
  GuidedLessonSession,
  GuidedPronunciationAttempt,
  PronunciationAttempt,
  TheoryBlock,
} from "../types";

type PlaybackState = {
  itemId: string;
  state: "preparing" | "playing";
  playbackId: string | null;
};

export function GuidedLessonSessionPage() {
  const { sessionId = "" } = useParams();
  const navigate = useNavigate();
  const [session, setSession] = useState<
    GuidedLessonSession | null | undefined
  >();
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirmAbandon, setConfirmAbandon] = useState(false);
  const [playback, setPlayback] = useState<PlaybackState | null>(null);
  const audio = useRef<HTMLAudioElement | null>(null);
  const playbackToken = useRef(0);
  const load = useCallback(async () => {
    setError(null);
    try {
      setSession(await getGuidedLessonSession(sessionId));
    } catch (value) {
      setError(message(value));
    }
  }, [sessionId]);
  const stopPlayback = useCallback(async () => {
    playbackToken.current += 1;
    audio.current?.pause();
    audio.current = null;
    const id = playback?.playbackId;
    if (id) await cancelGuidedLessonAudio(id);
    setPlayback(null);
  }, [playback]);
  useEffect(() => {
    void load();
    return () => {
      playbackToken.current += 1;
      audio.current?.pause();
    };
  }, [load]);
  const play = async (itemId: string) => {
    if (!session?.activeStage || playback) return;
    const token = ++playbackToken.current;
    setError(null);
    setPlayback({ itemId, state: "preparing", playbackId: null });
    try {
      const prepared = await prepareGuidedLessonAudio(
        session.id,
        session.activeStage.stageId,
        itemId,
      );
      if (token !== playbackToken.current) {
        await cancelGuidedLessonAudio(prepared.playbackId);
        return;
      }
      const player = new Audio(
        `data:${prepared.mimeType};base64,${prepared.audioBase64}`,
      );
      audio.current = player;
      setPlayback({
        itemId,
        state: "playing",
        playbackId: prepared.playbackId,
      });
      player.onended = () => {
        if (token !== playbackToken.current) return;
        audio.current = null;
        void completeGuidedLessonAudio(
          prepared.playbackId,
          session.id,
          session.activeStage!.stageId,
          itemId,
        )
          .then(setSession)
          .catch((value) => setError(message(value)))
          .finally(() => setPlayback(null));
      };
      player.onerror = () => {
        if (token !== playbackToken.current) return;
        void cancelGuidedLessonAudio(prepared.playbackId);
        audio.current = null;
        setPlayback(null);
        setError("Reference audio could not be played to completion.");
      };
      await player.play();
    } catch (value) {
      if (token === playbackToken.current) {
        setPlayback(null);
        setError(message(value));
      }
    }
  };
  const act = async (kind: "complete" | "skip") => {
    if (!session?.activeStage || busy) return;
    setBusy(true);
    setError(null);
    try {
      await stopPlayback();
      setSession(
        kind === "complete"
          ? await completeGuidedLessonStage(
              session.id,
              session.activeStage.stageId,
            )
          : await skipGuidedLessonStage(
              session.id,
              session.activeStage.stageId,
            ),
      );
    } catch (value) {
      setError(message(value));
    } finally {
      setBusy(false);
    }
  };
  const abandon = async () => {
    if (!session) return;
    setBusy(true);
    setError(null);
    try {
      await stopPlayback();
      await abandonGuidedLesson(session.id);
      navigate("/guided-lessons");
    } catch (value) {
      setError(message(value));
    } finally {
      setBusy(false);
    }
  };
  if (session === undefined && !error)
    return <LoadingState label="Restoring Guided Lesson…" />;
  if (error && !session)
    return <ErrorState message={error} onRetry={() => void load()} />;
  if (!session)
    return (
      <PageShell>
        <PageHeader
          title="Guided lesson session not found"
          description="This saved session does not exist."
        />
        <Link
          className="button-secondary inline-flex no-underline"
          to="/guided-lessons"
        >
          Back to Guided Lessons
        </Link>
      </PageShell>
    );
  if (session.status === "completed")
    return (
      <PageShell width="narrow">
        <section className="glass rounded-[28px] p-8 text-center">
          <div className="mx-auto grid h-14 w-14 place-items-center rounded-full bg-emerald-400/10 text-emerald-300">
            <Check size={28} />
          </div>
          <p className="eyebrow mt-5">Guided Lesson complete</p>
          <h1 className="page-title">{session.title}</h1>
          <p className="page-description">
            You completed all {session.stageCount} stages. There is no score, XP
            or analysis, and no pass mark or certification.
          </p>
          <Link
            className="button-primary mt-4 inline-flex no-underline"
            to="/guided-lessons"
          >
            Back to Guided Lessons
          </Link>
        </section>
      </PageShell>
    );
  if (session.status !== "in_progress")
    return (
      <PageShell width="narrow">
        <PageHeader
          eyebrow="Saved Guided Lesson"
          title={session.title}
          description={`This session is ${session.status}. Saved sessions cannot be changed.`}
        />
        <Link
          className="button-secondary inline-flex no-underline"
          to="/guided-lessons"
        >
          Back to Guided Lessons
        </Link>
      </PageShell>
    );
  const stage = session.activeStage;
  return (
    <PageShell width="standard">
      <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
        <Link
          to="/guided-lessons"
          className="muted inline-flex items-center gap-2 text-sm no-underline"
        >
          <ArrowLeft size={15} />
          Guided Lessons
        </Link>
        <button
          type="button"
          className="button-secondary"
          onClick={() => setConfirmAbandon(true)}
        >
          <X size={15} />
          Abandon Lesson
        </button>
      </div>
      <header className="glass rounded-[24px] p-5">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <p className="eyebrow">{session.cefrBand} · Guided Lesson</p>
            <h1 className="m-0 text-2xl">{session.title}</h1>
          </div>
          <strong aria-live="polite" className="text-sm">
            {session.progressPercent}% complete
          </strong>
        </div>
        <div
          className="mt-4 h-2 overflow-hidden rounded-full bg-white/[.06]"
          role="progressbar"
          aria-label="Lesson progress"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={session.progressPercent}
        >
          <div
            className="h-full bg-[var(--accent)] transition-[width]"
            style={{ width: `${session.progressPercent}%` }}
          />
        </div>
        <ol
          aria-label="Lesson stages"
          className="mt-4 flex gap-2 overflow-x-auto p-0"
        >
          {session.stages.map((item) => (
            <li
              key={item.stageId}
              aria-current={item.status === "active" ? "step" : undefined}
              className={`min-w-28 rounded-xl border px-3 py-2 text-xs ${item.status === "active" ? "border-[var(--accent)]/45 bg-[var(--accent)]/[.06]" : "border-white/10"}`}
            >
              <span className="muted">{item.sequenceIndex + 1}</span>
              <div className="mt-1 truncate">{item.title}</div>
            </li>
          ))}
        </ol>
      </header>
      {!stage ? (
        <section className="glass mt-5 rounded-[24px] p-6">
          <h2 className="section-title">Stage unavailable</h2>
          <p className="page-description">
            The saved stage cannot be rendered by this engine. Your session
            remains preserved.
          </p>
        </section>
      ) : (
        <section
          className="glass mt-5 rounded-[28px] p-5 md:p-8"
          aria-labelledby="active-stage-title"
        >
          <p className="eyebrow">
            Stage {stage.sequenceIndex + 1} of {session.stageCount}
          </p>
          <h2 id="active-stage-title" className="page-title">
            {stage.title}
          </h2>
          <p className="page-description">{stage.instructions}</p>
          <StageContent
            content={stage.content}
            session={session}
            playback={playback}
            play={play}
            stopPlayback={stopPlayback}
            update={setSession}
            reportError={setError}
          />
          {stage.content.kind !== "exercise" &&
            stage.content.kind !== "guided_conversation" && (
              <div className="mt-7 flex flex-wrap gap-3">
                <button
                  type="button"
                  className="button-primary"
                  disabled={busy || !!playback}
                  onClick={() => void act("complete")}
                >
                  {busy ? "Saving…" : "Continue"}
                </button>
                {!stage.required && (
                  <button
                    type="button"
                    className="button-secondary"
                    disabled={busy || !!playback}
                    onClick={() => void act("skip")}
                  >
                    Skip optional stage
                  </button>
                )}
              </div>
            )}
          {(stage.content.kind === "exercise" ||
            stage.content.kind === "guided_conversation") &&
            !stage.required && (
              <button
                type="button"
                className="button-secondary mt-5"
                disabled={busy}
                onClick={() => void act("skip")}
              >
                Skip optional stage
              </button>
            )}
          {error && (
            <div role="alert" className="mt-4 text-sm text-red-200">
              <p>{error}</p>
              <Link className="underline" to="/diagnostics">
                Open System Diagnostics
              </Link>
            </div>
          )}
        </section>
      )}
      <ConfirmationDialog
        open={confirmAbandon}
        title="Abandon this Guided Lesson?"
        description="Progress will be saved in history, but this session cannot be resumed after it is abandoned."
        confirmLabel="Abandon Lesson"
        danger
        busy={busy}
        onClose={() => setConfirmAbandon(false)}
        onConfirm={() => void abandon()}
      />
    </PageShell>
  );
}

function StageContent(props: {
  content: GuidedActiveContent;
  session: GuidedLessonSession;
  playback: PlaybackState | null;
  play: (id: string) => Promise<void>;
  stopPlayback: () => Promise<void>;
  update: (value: GuidedLessonSession) => void;
  reportError: (value: string | null) => void;
}) {
  const { content } = props;
  if (content.kind === "guided_conversation")
    return (
      <GuidedConversationStage
        content={content}
        session={props.session}
        stageId={props.session.activeStage!.stageId}
        update={props.update}
        reportError={props.reportError}
      />
    );
  if (content.kind === "theory")
    return <TheoryContent blocks={content.blocks} />;
  if (content.kind === "visual_vocabulary")
    return (
      <div className="mt-6 grid gap-4 sm:grid-cols-2">
        {content.items.map((item) => (
          <article
            key={item.itemId}
            className="rounded-2xl border border-white/10 bg-white/[.035] p-5"
          >
            <h3 className="m-0 text-xl text-[var(--accent)]">{item.term}</h3>
            <p className="mt-2 text-sm">{item.meaning}</p>
            <p className="muted mb-0 text-sm italic">“{item.example}”</p>
            {item.imageAssetId && (
              <p className="muted mb-0 mt-3 text-xs">
                Visual reference included in the local package.
              </p>
            )}
          </article>
        ))}
      </div>
    );
  if (content.kind === "listening")
    return (
      <div className="mt-6 grid gap-4">
        {content.segments.map((item) => {
          const active = props.playback?.itemId === item.segmentId;
          const revealed =
            !content.revealTextAfterFirstPlay ||
            item.completedPlaybackCount > 0;
          return (
            <article
              key={item.segmentId}
              className="rounded-2xl border border-white/10 p-5"
            >
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <h3 className="m-0 text-base">Listening {item.segmentId}</h3>
                  <p className="muted mb-0 mt-1 text-xs" aria-live="polite">
                    Completed plays: {item.completedPlaybackCount}
                  </p>
                </div>
                <button
                  className="button-secondary"
                  disabled={!!props.playback}
                  onClick={() => void props.play(item.segmentId)}
                >
                  <Play size={16} />
                  {active
                    ? props.playback?.state === "preparing"
                      ? "Preparing…"
                      : "Playing…"
                    : item.completedPlaybackCount
                      ? "Replay"
                      : "Play"}
                </button>
              </div>
              <p className="mt-4" aria-live="polite">
                {revealed
                  ? item.text
                  : "Text appears after the first complete playback."}
              </p>
            </article>
          );
        })}
      </div>
    );
  if (content.kind === "exercise")
    return (
      <GuidedExerciseStage
        stage={content.stage}
        session={props.session}
        stageId={props.session.activeStage!.stageId}
        update={props.update}
        reportError={props.reportError}
      />
    );
  return <PronunciationStage {...props} content={content} />;
}

function PronunciationStage({
  content,
  session,
  playback,
  play,
  stopPlayback,
  update,
  reportError,
}: {
  content: Extract<GuidedActiveContent, { kind: "repeat" | "speaking_check" }>;
  session: GuidedLessonSession;
  playback: PlaybackState | null;
  play: (id: string) => Promise<void>;
  stopPlayback: () => Promise<void>;
  update: (value: GuidedLessonSession) => void;
  reportError: (value: string | null) => void;
}) {
  const stage = session.activeStage!;
  const [selected, setSelected] = useState<string | null>(null);
  const onCapture = useCallback(
    async (audioBase64: string) => {
      if (!selected) return;
      reportError(null);
      update(
        await submitGuidedLessonPronunciation(
          session.id,
          stage.stageId,
          selected,
          audioBase64,
        ),
      );
    },
    [reportError, selected, session.id, stage.stageId, update],
  );
  const recorder = useGuidedLessonRecorder(onCapture);
  const record = async (id: string) => {
    await stopPlayback();
    setSelected(id);
    await recorder.start();
  };
  return (
    <div className="mt-6 grid gap-5">
      {content.targets.map((item) => {
        const id = item.targetId;
        const repeat = "text" in item;
        const text = repeat ? item.text : item.targetText;
        const attempts = item.attempts;
        const latest = attempts.at(-1);
        const referenceReady =
          !repeat || item.completedReferencePlaybackCount > 0;
        const recording = selected === id && recorder.state === "recording";
        const processing = selected === id && recorder.state === "processing";
        return (
          <article key={id} className="rounded-2xl border border-white/10 p-5">
            <p className="eyebrow">{repeat ? "Repeat" : "Speaking Check"}</p>
            {!repeat && <p className="muted text-sm">{item.instruction}</p>}
            <h3 className="break-words text-xl">{text}</h3>
            {item.hint && <p className="muted text-sm">Hint: {item.hint}</p>}
            <div className="flex flex-wrap gap-3">
              {repeat && (
                <button
                  className="button-secondary"
                  disabled={
                    !!playback ||
                    recorder.state === "recording" ||
                    recorder.state === "processing"
                  }
                  onClick={() => void play(id)}
                >
                  <Play size={16} />
                  {item.completedReferencePlaybackCount
                    ? "Replay reference"
                    : "Play reference"}
                </button>
              )}
              {recording ? (
                <button
                  className="button-primary"
                  onClick={() => void recorder.stop()}
                >
                  <Square size={16} />
                  Stop recording
                </button>
              ) : (
                <button
                  className="button-primary"
                  disabled={
                    !referenceReady ||
                    !!playback ||
                    recorder.state === "processing"
                  }
                  onClick={() => void record(id)}
                >
                  <Mic size={16} />
                  {latest ? "Try Again" : "Record"}
                </button>
              )}
            </div>
            {!referenceReady && (
              <p className="muted mt-3 text-xs">
                Listen to the complete reference before recording.
              </p>
            )}
            {processing && (
              <p role="status" className="mt-3 text-sm">
                Checking phrase and analyzing pronunciation locally…
              </p>
            )}
            {selected === id && recorder.error && (
              <p role="alert" className="mt-3 text-sm text-red-200">
                {recorder.error}
              </p>
            )}
            {attempts.length > 0 && (
              <div className="mt-4 grid gap-3">
                {attempts.map((attempt) => (
                  <GuidedAttemptCard
                    key={attempt.id}
                    attempt={attempt}
                    select={
                      attempt.status === "completed" && !attempt.selected
                        ? async () =>
                            update(
                              await selectGuidedLessonPronunciationAttempt(
                                session.id,
                                stage.stageId,
                                id,
                                attempt.id,
                              ),
                            )
                        : undefined
                    }
                  />
                ))}
              </div>
            )}
          </article>
        );
      })}
      <InlineNotice tone="info">
        There is no pass score. Any completed acoustic result can be selected;
        unsuccessful or cancelled analyses must be retried.
      </InlineNotice>
    </div>
  );
}

function GuidedAttemptCard({
  attempt,
  select,
}: {
  attempt: GuidedPronunciationAttempt;
  select?: () => Promise<void>;
}) {
  const [busy, setBusy] = useState(false);
  const result = attempt.result;
  return (
    <div
      className={`rounded-xl border p-4 ${attempt.selected ? "border-emerald-400/40 bg-emerald-400/[.05]" : "border-white/[.08] bg-white/[.03]"}`}
    >
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <strong>Attempt {attempt.attemptIndex}</strong>
          <div className="muted text-xs">
            {attempt.status.replace("_", " ")}
          </div>
        </div>
        {result?.status === "completed" && (
          <div className="text-right">
            <strong>{Math.round(result.overallScore ?? 0)}/100</strong>
            <div className="muted text-xs">Confidence: {result.confidence}</div>
          </div>
        )}
      </div>
      {result && <CompactResult result={result} />}
      <div className="mt-3 flex gap-2">
        {attempt.selected && (
          <span className="text-sm text-emerald-300">
            <Check size={15} className="inline" /> Selected
          </span>
        )}
        {select && (
          <button
            className="button-secondary"
            disabled={busy}
            onClick={() => {
              setBusy(true);
              void select().finally(() => setBusy(false));
            }}
          >
            Continue with this attempt
          </button>
        )}
      </div>
    </div>
  );
}

function CompactResult({ result }: { result: PronunciationAttempt }) {
  if (result.status !== "completed")
    return (
      <p className="muted mb-0 mt-3 text-sm">
        No selectable score was produced. Try again.
      </p>
    );
  return (
    <div className="mt-3 flex flex-wrap gap-2" aria-label="Word scores">
      {result.words.map((word) => (
        <span
          key={word.index}
          className="rounded-lg bg-black/20 px-2 py-1 text-xs"
        >
          {word.word} <strong>{Math.round(word.score)}</strong>
        </span>
      ))}
    </div>
  );
}
function TheoryContent({ blocks }: { blocks: TheoryBlock[] }) {
  return (
    <div className="mt-6 space-y-4">
      {blocks.map((block, index) => {
        if (block.type === "paragraph")
          return (
            <p key={index} className="leading-7">
              {block.text}
            </p>
          );
        if (block.type === "bullet_list")
          return (
            <ul key={index} className="space-y-2 pl-5">
              {block.items?.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          );
        if (block.type === "example")
          return (
            <figure key={index} className="rounded-2xl bg-white/[.04] p-5">
              <blockquote className="m-0 text-lg">{block.english}</blockquote>
              {block.explanation && (
                <figcaption className="muted mt-2 text-sm">
                  {block.explanation}
                </figcaption>
              )}
            </figure>
          );
        return (
          <aside
            key={index}
            className="rounded-2xl border border-[var(--accent)]/20 bg-[var(--accent)]/[.04] p-5"
          >
            {block.title && <h3 className="mt-0 text-base">{block.title}</h3>}
            <p className="mb-0 text-sm leading-6">{block.text}</p>
          </aside>
        );
      })}
    </div>
  );
}
function message(value: unknown) {
  return value instanceof Error ? value.message : String(value);
}
