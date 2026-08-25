// @vitest-environment jsdom

import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { StudentLearningProfile } from '../types'

const mocks = vi.hoisted(() => ({ get: vi.fn(), update: vi.fn() }))
vi.mock('../services/native', () => ({ getStudentLearningProfile: mocks.get, updateStudentLearningProfile: mocks.update }))
import { ProfilePage } from './ProfilePage'

const empty: StudentLearningProfile = { schemaVersion: 1, currentPlacement: null, targetLevel: null, learningGoals: [], defaultLessonDifficulty: 'standard', useProfileInLessons: true }
function page() { return render(<MemoryRouter><ProfilePage /></MemoryRouter>) }
beforeEach(() => { mocks.get.mockResolvedValue(empty); mocks.update.mockImplementation(async (request) => ({ ...empty, ...request })) })
afterEach(() => { cleanup(); vi.clearAllMocks() })

describe('ProfilePage', () => {
  it('shows the honest no-placement state and still exposes editable preferences', async () => {
    page()
    expect(await screen.findByText('Not assessed')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Take Placement Test' })).toHaveAttribute('href', '/placement')
    expect(screen.getByLabelText('Target CEFR')).toHaveValue('')
    expect(screen.getByRole('checkbox', { name: /Use Student Profile/ })).toBeChecked()
  })

  it('renders the real current placement as read-only with result links', async () => {
    mocks.get.mockResolvedValue({ ...empty, currentPlacement: { attemptId: 'attempt-real', estimatedLevel: 'B1', confidence: 'medium', assessedAt: '2026-08-21T12:00:00Z' } })
    page()
    expect((await screen.findAllByText('B1')).some((element) => element.tagName === 'STRONG')).toBe(true)
    expect(screen.getByText('Medium')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'View Placement Result' })).toHaveAttribute('href', '/placement/results/attempt-real')
    expect(screen.queryByLabelText(/current estimated/i)).not.toBeInTheDocument()
  })

  it('enforces three keyboard-accessible goals and saves only editable fields', async () => {
    page(); await screen.findByText('Learning Goals')
    for (const label of ['General Fluency', 'Travel English', 'Speaking Confidence']) fireEvent.click(screen.getByRole('button', { name: label }))
    expect(screen.getByRole('button', { name: 'Academic English' })).toBeDisabled()
    fireEvent.change(screen.getByLabelText('Target CEFR'), { target: { value: 'C1' } })
    fireEvent.click(screen.getByRole('button', { name: 'Challenging' }))
    fireEvent.click(screen.getByRole('button', { name: 'Save Profile' }))
    await waitFor(() => expect(mocks.update).toHaveBeenCalledWith({ targetLevel: 'C1', learningGoals: ['general_fluency', 'travel_english', 'speaking_confidence'], defaultLessonDifficulty: 'challenging', useProfileInLessons: true }))
    expect(await screen.findByText('Profile saved locally.')).toBeInTheDocument()
  })
})
