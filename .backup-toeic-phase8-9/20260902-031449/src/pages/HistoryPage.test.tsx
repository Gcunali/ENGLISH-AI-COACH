// @vitest-environment jsdom

import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { historyItem } from '../test/learningFixtures'

const mocks = vi.hoisted(() => ({ useHistoryData: vi.fn() }))
vi.mock('../hooks/useLearningData', () => ({ useHistoryData: mocks.useHistoryData }))
import { HistoryPage } from './HistoryPage'

const reload = vi.fn()

describe('HistoryPage', () => {
  beforeEach(() => vi.clearAllMocks())
  afterEach(cleanup)

  it('renders the history empty state', () => {
    mocks.useHistoryData.mockReturnValue({ data: { items: [], total: 0, limit: 20, offset: 0 }, loading: false, error: null, reload })
    render(<MemoryRouter><HistoryPage /></MemoryRouter>)
    expect(screen.getByText('No lessons found')).toBeInTheDocument()
    expect(screen.getByText(/lessons will appear here/)).toBeInTheDocument()
  })

  it('renders factual lesson and analysis statuses', () => {
    mocks.useHistoryData.mockReturnValue({ data: { items: [historyItem], total: 1, limit: 20, offset: 0 }, loading: false, error: null, reload })
    render(<MemoryRouter><HistoryPage /></MemoryRouter>)
    expect(screen.getByText('Conversation lesson')).toBeInTheDocument()
    expect(screen.getByText('Lesson: Completed')).toBeInTheDocument()
    expect(screen.getByText('Analysis: Completed')).toBeInTheDocument()
    expect(screen.getByText('81')).toBeInTheDocument()
  })

  it('changes filters and paginates with predictable offset', () => {
    mocks.useHistoryData.mockReturnValue({ data: { items: [historyItem], total: 25, limit: 20, offset: 0 }, loading: false, error: null, reload })
    render(<MemoryRouter><HistoryPage /></MemoryRouter>)
    fireEvent.click(screen.getByRole('button', { name: 'Interrupted' }))
    expect(mocks.useHistoryData).toHaveBeenLastCalledWith('interrupted', 20, 0)
    fireEvent.click(screen.getByRole('button', { name: /Next/ }))
    expect(mocks.useHistoryData).toHaveBeenLastCalledWith('interrupted', 20, 20)
  })
})
