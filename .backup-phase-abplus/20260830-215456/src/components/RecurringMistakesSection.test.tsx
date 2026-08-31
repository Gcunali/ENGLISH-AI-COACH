// @vitest-environment jsdom

import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { recurringMistake, recurringMistakeDetails } from '../test/learningFixtures'

const mocks = vi.hoisted(() => ({ useRecurringMistakesData: vi.fn(), getRecurringMistake: vi.fn() }))
vi.mock('../hooks/useLearningData', () => ({ useRecurringMistakesData: mocks.useRecurringMistakesData }))
vi.mock('../services/native', () => ({ getRecurringMistake: mocks.getRecurringMistake }))
import { RecurringMistakesSection } from './RecurringMistakesSection'

const reload = vi.fn()

describe('RecurringMistakesSection', () => {
  beforeEach(() => { vi.clearAllMocks(); mocks.getRecurringMistake.mockResolvedValue(recurringMistakeDetails) })
  afterEach(cleanup)

  it('states factually that no cross-lesson recurrence is confirmed', () => {
    mocks.useRecurringMistakesData.mockReturnValue({ data: [], loading: false, error: null, reload })
    render(<MemoryRouter><RecurringMistakesSection /></MemoryRouter>)
    expect(screen.getByText('No recurring mistakes have been confirmed across multiple lessons yet.')).toBeInTheDocument()
  })

  it('shows confirmed counts, category and expands only real occurrences', async () => {
    mocks.useRecurringMistakesData.mockReturnValue({ data: [recurringMistake], loading: false, error: null, reload })
    render(<MemoryRouter><RecurringMistakesSection /></MemoryRouter>)
    expect(screen.getByText('Detected across 2 lessons', { exact: false })).toBeInTheDocument()
    expect(screen.getByText('Preposition')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: /Preposition: "terrible at cooking"/ }))
    await waitFor(() => expect(mocks.getRecurringMistake).toHaveBeenCalledWith('mistake-1'))
    expect(screen.getAllByText('YOU SAID')).toHaveLength(2)
    expect(screen.getAllByRole('link', { name: 'View lesson' })[0]).toHaveAttribute('href', '/history/lesson-1')
  })

  it('renders multiple confirmed mistakes without hiding their labels', () => {
    mocks.useRecurringMistakesData.mockReturnValue({ data: [recurringMistake, { ...recurringMistake, id: 'mistake-2', category: 'verb_tense', title: 'Verb tense: "played"' }], loading: false, error: null, reload })
    render(<MemoryRouter><RecurringMistakesSection /></MemoryRouter>)
    expect(screen.getByText('Preposition: "terrible at cooking"')).toBeInTheDocument()
    expect(screen.getByText('Verb tense: "played"')).toBeInTheDocument()
  })
})
