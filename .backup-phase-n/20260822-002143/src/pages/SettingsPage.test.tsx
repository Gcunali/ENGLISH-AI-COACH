// @vitest-environment jsdom

import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { studentLearningSummary } from '../test/learningFixtures'

const native = vi.hoisted(() => ({
  getStudentLearningSummary: vi.fn(),
  getLearningMemoryEnabled: vi.fn(),
  setLearningMemoryEnabled: vi.fn(),
}))

vi.mock('../services/native', () => native)

import { SettingsPage } from './SettingsPage'

beforeEach(() => {
  native.getStudentLearningSummary.mockResolvedValue(studentLearningSummary)
  native.getLearningMemoryEnabled.mockResolvedValue(true)
  native.setLearningMemoryEnabled.mockImplementation(async (enabled: boolean) => enabled)
})
afterEach(() => { cleanup(); vi.clearAllMocks() })

describe('SettingsPage learning memory', () => {
  it('shows loading while the local summary is pending', () => {
    native.getStudentLearningSummary.mockReturnValue(new Promise(() => undefined))
    render(<SettingsPage />)
    expect(screen.getByText('Loading your local learning summary…')).toBeInTheDocument()
  })

  it('shows a persisted off setting', async () => {
    native.getLearningMemoryEnabled.mockResolvedValue(false)
    render(<SettingsPage />)
    const toggle = await screen.findByRole('switch')
    await waitFor(() => expect(toggle).toHaveAttribute('aria-checked', 'false'))
    expect(toggle).toHaveTextContent('Off')
  })

  it('shows persisted pedagogical memory and honest empty recurring state', async () => {
    render(<SettingsPage />)
    expect(await screen.findByText('Built from 2 analyzed lessons.')).toBeInTheDocument()
    expect(screen.getByText('Keeps the conversation moving')).toBeInTheDocument()
    expect(screen.getByText('preposition: Use natural prepositions')).toBeInTheDocument()
    expect(screen.getByText('No recurring mistakes have been confirmed across multiple lessons.')).toBeInTheDocument()
    expect(screen.getByText('terrible at — very bad at something (learning)')).toBeInTheDocument()
    expect(screen.getByRole('switch')).toHaveAttribute('aria-checked', 'true')
  })

  it('persists the toggle immediately and restores it if persistence fails', async () => {
    render(<SettingsPage />)
    const toggle = await screen.findByRole('switch')
    fireEvent.click(toggle)
    await waitFor(() => expect(native.setLearningMemoryEnabled).toHaveBeenCalledWith(false))
    expect(toggle).toHaveAttribute('aria-checked', 'false')

    native.setLearningMemoryEnabled.mockRejectedValueOnce(new Error('write failed'))
    fireEvent.click(toggle)
    expect(await screen.findByRole('alert')).toHaveTextContent('write failed')
    expect(toggle).toHaveAttribute('aria-checked', 'false')
  })

  it('explains the empty state without inventing history', async () => {
    native.getStudentLearningSummary.mockResolvedValue({
      ...studentLearningSummary,
      analyzedLessonCount: 0,
      completedLessonCount: 0,
      recentStrengths: [], currentFocusAreas: [], confirmedRecurringMistakes: [],
      recentVocabulary: [], nextLessonRecommendations: [], latestPerformanceSnapshot: null,
    })
    render(<SettingsPage />)
    expect(await screen.findByText('No learning memory yet')).toBeInTheDocument()
  })
})
