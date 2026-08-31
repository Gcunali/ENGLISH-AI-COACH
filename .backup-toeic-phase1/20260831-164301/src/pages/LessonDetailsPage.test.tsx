// @vitest-environment jsdom

import '@testing-library/jest-dom/vitest'
import { cleanup, render, screen, within } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { details } from '../test/learningFixtures'

const mocks = vi.hoisted(() => ({ useLessonDetailsData: vi.fn(), retryLessonAnalysis: vi.fn() }))
vi.mock('../hooks/useLearningData', () => ({ useLessonDetailsData: mocks.useLessonDetailsData }))
vi.mock('../services/native', () => ({ retryLessonAnalysis: mocks.retryLessonAnalysis }))
import { LessonDetailsPage } from './LessonDetailsPage'

const reload = vi.fn()
function renderPage() {
  return render(<MemoryRouter initialEntries={['/history/lesson-1']}><Routes><Route path="/history/:lessonId" element={<LessonDetailsPage />} /></Routes></MemoryRouter>)
}

describe('LessonDetailsPage', () => {
  beforeEach(() => vi.clearAllMocks())
  afterEach(cleanup)

  it('loads the exact ordered transcript, correction relation and persisted analysis', () => {
    mocks.useLessonDetailsData.mockReturnValue({ data: details, loading: false, error: null, reload })
    renderPage()
    const transcript = screen.getByRole('region', { name: 'Full transcript' })
    const text = transcript.textContent ?? ''
    expect(text.indexOf('I am terrible cooking.')).toBeLessThan(text.indexOf("You can say: I'm terrible at cooking."))
    expect(within(transcript).getByText('Correction')).toBeInTheDocument()
    expect(screen.getByText('Boa abertura e engajamento')).toBeInTheDocument()
    expect(screen.getAllByText('terrible at', { exact: false }).length).toBeGreaterThan(0)
    expect(screen.getByText('Not evaluated yet', { exact: false })).toBeInTheDocument()
    expect(screen.getByText('Student profile context was not recorded for this lesson.')).toBeInTheDocument()
  })

  it('treats an interrupted lesson without analysis as factual, not an error', () => {
    mocks.useLessonDetailsData.mockReturnValue({ data: { ...details, lesson: { ...details.lesson, status: 'interrupted' }, analysis: null }, loading: false, error: null, reload })
    renderPage()
    expect(screen.getByText('Lesson interrupted')).toBeInTheDocument()
    expect(screen.getByText(/Analysis is unavailable/)).toBeInTheDocument()
  })

  it('renders a not-found state without crashing', () => {
    mocks.useLessonDetailsData.mockReturnValue({ data: null, loading: false, error: null, reload })
    renderPage()
    expect(screen.getByText('Lesson not found')).toBeInTheDocument()
  })

  it('shows structured profile metadata without presenting it as a lesson score', () => {
    mocks.useLessonDetailsData.mockReturnValue({ data: { ...details, studentProfileSnapshot: { lessonId: details.lesson.id, profileSchemaVersion: 1, profileContextVersion: 1, contextEnabled: true, placementAttemptId: 'placement-1', estimatedCefrLevel: 'B1', placementConfidence: 'medium', targetCefrLevel: 'B2', learningGoals: ['general_fluency', 'speaking_confidence'], lessonDifficulty: 'challenging', createdAt: details.lesson.startedAt } }, loading: false, error: null, reload })
    renderPage()
    const setup = screen.getByRole('region', { name: 'Profile at lesson start' })
    expect(within(setup).getByText('B1')).toBeInTheDocument()
    expect(within(setup).getByText('B2')).toBeInTheDocument()
    expect(within(setup).getByText('General Fluency, Speaking Confidence')).toBeInTheDocument()
    expect(within(setup).queryByText(/lesson level/i)).not.toBeInTheDocument()
  })
})
