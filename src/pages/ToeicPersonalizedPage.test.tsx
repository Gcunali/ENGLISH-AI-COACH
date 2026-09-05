// @vitest-environment jsdom
import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ToeicPersonalizedPage } from "./ToeicPersonalizedPage";

const api = vi.hoisted(() => ({
  getToeicPersonalizedDashboard: vi.fn(),
  getToeicPersonalizedPractice: vi.fn(),
  setToeicTargetScore: vi.fn(),
  startToeicPersonalizedPractice: vi.fn(),
}));
vi.mock("../services/native", () => api);

const dashboard = {
  targetScore: 750, latestListeningEstimate: 405, latestReadingEstimate: 370,
  latestTotalEstimate: 775, latestRangeLow: 720, latestRangeHigh: 830, estimatedGap: -25,
  exposure: { uniqueSeen: 200, bankItems: 600, totalAnswers: 210, unseen: 400, repeatedItems: 10 },
  weaknesses: [{ partNumber: 5, skill: "Word Forms", correct: 2, total: 6, accuracy: 33, label: "Priority", sufficientSample: true, lastSeenAt: "now" }],
  priorities: [{ rank: 1, partNumber: 5, skill: "Word Forms", reason: "33% recent-weighted accuracy across 6 first attempts.", route: "/toeic/personalized" }],
  recommendations: [{ title: "Smart Practice · 15 questions", description: "Focus on Part 5.", route: "/toeic/personalized" }],
  trends: [{ sessionId: "full-a", family: "A", completedAt: "2026-09-02T12:00:00Z", listeningRaw: 79, readingRaw: 71, totalRaw: 150, listeningEstimate: 405, readingEstimate: 370, totalEstimate: 775 }],
  activePractice: null, recentPractice: [],
};
const practice = { sessionId: "practice-1", kind: "smart", requestedCount: 15, answeredCount: 0, correctCount: null, accuracy: null, status: "in_progress", focus: ["Word Forms"], createdAt: "now", completedAt: null, steps: [{ stepNumber: 1, partNumber: 5, formId: "toeic-part5-form-a", sessionId: "child", quota: 15, answered: 0, correct: 0, status: "in_progress", route: "/toeic/part5/session/child?toeicPractice=practice-1&limit=15" }] };

afterEach(cleanup);
beforeEach(() => { vi.clearAllMocks(); api.getToeicPersonalizedDashboard.mockResolvedValue(dashboard); api.setToeicTargetScore.mockResolvedValue({ ...dashboard, targetScore: 850 }); api.startToeicPersonalizedPractice.mockResolvedValue(practice); api.getToeicPersonalizedPractice.mockResolvedValue(practice); });

describe("TOEIC personalized preparation", () => {
  it("shows target, unofficial range, priorities and exposure", async () => {
    render(<MemoryRouter><ToeicPersonalizedPage /></MemoryRouter>);
    expect(await screen.findByText("775")).toBeInTheDocument();
    expect(screen.getByText("Unofficial range 720–830")).toBeInTheDocument();
    expect(screen.getAllByText("Part 5 · Word Forms")).toHaveLength(2);
    expect(screen.getByText("200/600")).toBeInTheDocument();
  });
  it("persists a target preset and starts a frozen smart-practice parent", async () => {
    render(<MemoryRouter initialEntries={["/toeic/personalized"]}><Routes><Route path="/toeic/personalized" element={<ToeicPersonalizedPage />} /><Route path="/toeic/personalized/:id" element={<ToeicPersonalizedPage />} /></Routes></MemoryRouter>);
    fireEvent.click(await screen.findByRole("button", { name: "850" }));
    await waitFor(() => expect(api.setToeicTargetScore).toHaveBeenCalledWith(850));
    fireEvent.click(screen.getByText("Practice My Weak Areas"));
    await waitFor(() => expect(api.startToeicPersonalizedPractice).toHaveBeenCalledWith("smart", 15));
    expect(await screen.findByText("Practice in progress")).toBeInTheDocument();
  });
});
