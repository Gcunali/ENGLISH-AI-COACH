// @vitest-environment jsdom

import '@testing-library/jest-dom/vitest'
import { cleanup, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { dashboard, memorySummary } from '../test/learningFixtures'
import { useConversationStore } from '../stores/conversation'

const mocks = vi.hoisted(() => ({
  useDashboardData: vi.fn(),
  useLearningMemorySummaryData: vi.fn(),
  useLocalVoiceEngine: vi.fn(),
  useLessonAnalysis: vi.fn(),
  probeLocalVoiceEngine: vi.fn(),
}))

vi.mock('../hooks/useLearningData', () => ({ useDashboardData: mocks.useDashboardData, useLearningMemorySummaryData: mocks.useLearningMemorySummaryData }))
vi.mock('../hooks/useLocalVoiceEngine', () => ({ useLocalVoiceEngine: mocks.useLocalVoiceEngine }))
vi.mock('../hooks/useLessonAnalysis', () => ({ useLessonAnalysis: mocks.useLessonAnalysis }))
vi.mock('../services/native', () => ({ probeLocalVoiceEngine: mocks.probeLocalVoiceEngine }))

import { DashboardPage } from './DashboardPage'

const reload = vi.fn()

describe('DashboardPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useConversationStore.getState().reset()
    mocks.useLocalVoiceEngine.mockReturnValue({ engineState: 'stopped', startLesson: vi.fn(), endLesson: vi.fn() })
    mocks.useLessonAnalysis.mockReturnValue({ analysis: null, status: null, error: null, retry: vi.fn() })
    mocks.probeLocalVoiceEngine.mockResolvedValue({ offlineReady: false, whisper: {}, ollama: {}, piper: {} })
    mocks.useLearningMemorySummaryData.mockReturnValue({ data: memorySummary, loading: false, error: null, reload })
  })
  afterEach(cleanup)

  it('renders a loading state without fake zeros', () => {
    mocks.useDashboardData.mockReturnValue({ data: null, loading: true, error: null, reload })
    render(<MemoryRouter><DashboardPage /></MemoryRouter>)
    expect(screen.getByText(/Loading dashboard/)).toBeInTheDocument()
    expect(screen.queryByText('Average score')).not.toBeInTheDocument()
  })

  it('renders the factual empty state while keeping Start Lesson', () => {
    mocks.useDashboardData.mockReturnValue({ data: { ...dashboard, totalLessons: 0, completedLessons: 0, totalPracticeSeconds: null, totalStudentTurns: 0, totalCorrections: 0, analyzedLessons: 0, averageOverallScore: null, latestLesson: null, latestAnalyzedLesson: null, latestRecommendation: null }, loading: false, error: null, reload })
    render(<MemoryRouter><DashboardPage /></MemoryRouter>)
    expect(screen.getByText('No lessons yet')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Start Lesson' })).toHaveAttribute('href', '/lesson/new')
    expect(screen.getAllByText('Not available').length).toBeGreaterThan(0)
  })

  it('renders persisted totals, latest scores and recommendation', () => {
    mocks.useDashboardData.mockReturnValue({ data: dashboard, loading: false, error: null, reload })
    render(<MemoryRouter><DashboardPage /></MemoryRouter>)
    for (const text of ['Total lessons', 'Practice time', 'Average score', 'Latest lesson', 'Current performance', 'Fluency', '85', 'Praticar preposições.']) {
      expect(screen.getAllByText(text, { exact: false }).length).toBeGreaterThan(0)
    }
    expect(screen.getByRole('link', { name: 'View lesson' })).toHaveAttribute('href', '/history/lesson-1')
    expect(screen.getByText('Vocabulary tracked')).toBeInTheDocument()
    expect(screen.getByText('Recurring mistakes')).toBeInTheDocument()
  })
})
