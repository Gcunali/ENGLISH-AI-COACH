// @vitest-environment jsdom
import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import * as native from "../services/native";
import type { GuidedLessonAnalysis, GuidedLessonSession } from "../types";
import { InteractiveLessonAnalysisStage } from "./InteractiveLessonAnalysisStage";

vi.mock("../services/native", () => ({
  getGuidedLessonAnalysis: vi.fn(),
  analyzeGuidedLesson: vi.fn(),
  retryGuidedLessonConversationAnalysis: vi.fn(),
  finalizeGuidedLessonAnalysis: vi.fn(),
  getGuidedLessonSession: vi.fn(),
}));

const session: GuidedLessonSession = {
  id: "guided-1", lessonId: "cafe", contentVersion: 1, title: "Ordering at a Café", cefrBand: "A1",
  status: "in_progress", currentStageIndex: 2, stageCount: 3, progressPercent: 66,
  stages: [
    { stageId: "theory", sequenceIndex: 0, stageType: "theory", title: "Theory", required: true, status: "completed", attemptCount: 1 },
    { stageId: "practice", sequenceIndex: 1, stageType: "exercise", title: "Practice", required: true, status: "completed", attemptCount: 1 },
    { stageId: "analysis", sequenceIndex: 2, stageType: "analysis", title: "Lesson Review", required: true, status: "active", attemptCount: 0 },
  ],
  activeStage: { stageId: "analysis", sequenceIndex: 2, stageType: "analysis", title: "Lesson Review", instructions: "Review your practice results.", required: true, content: { kind: "analysis" } },
  startedAt: "2026-08-24", completedAt: null, abandonedAt: null,
};

function fixture(status: "completed" | "partial" = "completed"): GuidedLessonAnalysis {
  return {
    id: "analysis-1", sessionId: session.id, stageId: "analysis", status,
    conversationStatus: status === "partial" ? "unavailable" : "completed",
    evidenceHash: "a".repeat(64), finalizedAt: null,
    errorCode: status === "partial" ? "conversation_evaluator_unavailable" : null,
    result: {
      schemaVersion: 1, analysisId: "analysis-1", lessonId: "cafe", contentVersion: 1, status,
      generatedAt: "2026-08-24T22:00:00Z",
      participation: {
        requiredStageCount: 2, completedRequiredStageCount: 2, skippedOptionalStageCount: 0,
        vocabularyItemCount: 4, listening: { segmentCount: 2, listenedSegmentCount: 2, totalPlaybackCount: 3 },
        stageStatus: [{ stageId: "theory", stageType: "theory", required: true, status: "completed", completedAt: "now" }],
      },
      conversation: status === "partial" ? {
        status: "unavailable", scores: null, goalProgress: null, strengths: [], focusAreas: [], summary: null,
      } : {
        status: "completed", scores: { grammar: 0, vocabulary: 84, fluency: 81, interaction: 86 }, goalProgress: "strong",
        strengths: [{ text: "Uses polite requests naturally.", studentTurnSequences: [2] }],
        focusAreas: [{ text: "Link sentences more naturally.", studentTurnSequences: [4] }],
        summary: "The learner sustained the exchange.",
      },
      exercises: { status: "completed", exerciseCount: 1, selectedAttemptCount: 1, selectedCorrectCount: 0, selectedIncorrectCount: 1, totalAttemptCount: 3, accuracyPercent: 0 },
      pronunciation: { status: "completed", selectedPhraseCount: 1, totalAttemptCount: 2, scoresAvailable: 1, meanAcousticMatch: 42, minimumAcousticMatch: 42, maximumAcousticMatch: 42, lowConfidenceCount: 1, issueSummary: [{ phone: "θ", selectedAttemptCount: 1, meanScore: 38, hint: "Keep airflow moving." }] },
      practicedObjectives: ["Order a drink politely."],
    },
  };
}

describe("Interactive Lesson Analysis stage", () => {
  afterEach(cleanup);
  beforeEach(() => vi.resetAllMocks());

  it("does not start analysis when the stage opens", async () => {
    vi.mocked(native.getGuidedLessonAnalysis).mockResolvedValue(null);
    render(<InteractiveLessonAnalysisStage session={session} stageId="analysis" updateSession={vi.fn()} />);
    expect(await screen.findByRole("button", { name: "Analyze Lesson" })).toBeEnabled();
    expect(native.analyzeGuidedLesson).not.toHaveBeenCalled();
  });

  it("renders separated zero and low results without overall, CEFR, pass or mastery copy", async () => {
    vi.mocked(native.getGuidedLessonAnalysis).mockResolvedValue(fixture());
    render(<InteractiveLessonAnalysisStage session={session} stageId="analysis" updateSession={vi.fn()} />);
    expect(await screen.findByLabelText("Grammar: 0 out of 100")).toBeInTheDocument();
    expect(screen.getByText("0%", { selector: "strong" })).toBeInTheDocument();
    expect(screen.getByText("42", { selector: "strong" })).toBeInTheDocument();
    expect(screen.getByText("Objectives practiced")).toBeInTheDocument();
    const text = document.body.textContent ?? "";
    expect(text).not.toMatch(/overall english|final english|objectives mastered|passed|failed/i);
    expect(text).toContain("not a CEFR assessment");
  });

  it("keeps deterministic sections visible when conversation feedback is partial", async () => {
    vi.mocked(native.getGuidedLessonAnalysis).mockResolvedValue(fixture("partial"));
    render(<InteractiveLessonAnalysisStage session={session} stageId="analysis" updateSession={vi.fn()} />);
    expect(await screen.findByText(/rest of your lesson results are ready/i)).toBeInTheDocument();
    expect(screen.getByText("0%", { selector: "strong" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Retry Conversation Feedback/i })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Finish Guided Lesson" })).toBeEnabled();
  });

  it("analyzes only after a click and prevents a double request", async () => {
    vi.mocked(native.getGuidedLessonAnalysis).mockResolvedValue(null);
    vi.mocked(native.analyzeGuidedLesson).mockResolvedValue(fixture());
    render(<InteractiveLessonAnalysisStage session={session} stageId="analysis" updateSession={vi.fn()} />);
    const button = await screen.findByRole("button", { name: "Analyze Lesson" });
    fireEvent.click(button);
    fireEvent.click(button);
    await waitFor(() => expect(native.analyzeGuidedLesson).toHaveBeenCalledTimes(1));
    expect(await screen.findByText("Your practice results")).toBeInTheDocument();
  });
});
