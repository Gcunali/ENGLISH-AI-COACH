// @vitest-environment jsdom

import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { progress } from '../test/learningFixtures'

const mocks = vi.hoisted(() => ({ useProgressData: vi.fn(), useRecurringMistakesData: vi.fn() }))
vi.mock('../hooks/useLearningData', () => ({ useProgressData: mocks.useProgressData, useRecurringMistakesData: mocks.useRecurringMistakesData }))
vi.mock('../hooks/useGamification', () => ({ useGamificationOverview: () => ({ data: { totalXp: 0, practiceLevel: 1, currentStreakDays: 0, totalPracticeMinutes: 0, weeklyGoal: { practicedMinutes: 0, goalMinutes: 90, progressPercent: 0, reached: false }, unlockedAchievementCount: 0, totalAchievementCount: 11 }, loading: false, error: null, reload }) }))
import { ProgressPage } from './ProgressPage'

const reload = vi.fn()

describe('ProgressPage', () => {
  beforeEach(() => { vi.clearAllMocks(); mocks.useRecurringMistakesData.mockReturnValue({ data: [], loading: false, error: null, reload }) })
  afterEach(cleanup)

  it('renders the zero-analysis empty state', () => {
    mocks.useProgressData.mockReturnValue({ data: { analyzedLessonCount: 0, averages: null, strongestAreas: [], focusAreas: [], points: [], latestRecommendation: null }, loading: false, error: null, reload })
    render(<MemoryRouter><ProgressPage /></MemoryRouter>)
    expect(screen.getByText('No analyzed lessons yet')).toBeInTheDocument()
  })

  it('renders one real point without claiming a trend', () => {
    mocks.useProgressData.mockReturnValue({ data: progress, loading: false, error: null, reload })
    render(<MemoryRouter><ProgressPage /></MemoryRouter>)
    expect(screen.getByRole('note')).toHaveTextContent('no trend is inferred')
    expect(screen.getByText('Average Grammar')).toBeInTheDocument()
    expect(screen.getAllByText('81').length).toBeGreaterThan(0)
    expect(screen.getByText('Pronunciation is not included', { exact: false })).toBeInTheDocument()
  })

  it('renders multiple chronological points and dimension selection', () => {
    const second = { ...progress.points[0], lessonId: 'lesson-2', date: '2026-08-18T19:44:27.388Z', overall: 85, grammar: 78 }
    mocks.useProgressData.mockReturnValue({ data: { ...progress, analyzedLessonCount: 2, points: [progress.points[0], second] }, loading: false, error: null, reload })
    render(<MemoryRouter><ProgressPage /></MemoryRouter>)
    expect(screen.queryByRole('note')).not.toBeInTheDocument()
    expect(screen.getAllByLabelText(/Lesson [12]/).length).toBeGreaterThanOrEqual(4)
    fireEvent.change(screen.getByLabelText('Dimension'), { target: { value: 'fluency' } })
    expect(screen.getByRole('heading', { name: 'Fluency over time' })).toBeInTheDocument()
  })
})
