// @vitest-environment jsdom

import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { vocabularyPage, vocabularySummary } from '../test/learningFixtures'

const mocks = vi.hoisted(() => ({ useVocabularyData: vi.fn(), useVocabularySummaryData: vi.fn() }))
vi.mock('../hooks/useLearningData', () => ({ useVocabularyData: mocks.useVocabularyData, useVocabularySummaryData: mocks.useVocabularySummaryData }))
import { VocabularyPage } from './VocabularyPage'

const reload = vi.fn()

describe('VocabularyPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.useVocabularySummaryData.mockReturnValue({ data: vocabularySummary, loading: false, error: null, reload })
    mocks.useVocabularyData.mockReturnValue({ data: vocabularyPage, loading: false, error: null, reload })
  })
  afterEach(cleanup)

  it('renders loading and the factual empty state', () => {
    mocks.useVocabularySummaryData.mockReturnValue({ data: null, loading: true, error: null, reload })
    mocks.useVocabularyData.mockReturnValue({ data: { items: [], total: 0, limit: 25, offset: 0 }, loading: false, error: null, reload })
    render(<MemoryRouter><VocabularyPage /></MemoryRouter>)
    expect(screen.getByText('Loading vocabulary summary…')).toBeInTheDocument()
    expect(screen.getByText('No vocabulary yet')).toBeInTheDocument()
  })

  it('renders real vocabulary and links to details and its lesson', () => {
    render(<MemoryRouter><VocabularyPage /></MemoryRouter>)
    expect(screen.getByText('terrible at')).toBeInTheDocument()
    expect(screen.getByText('muito ruim em')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /terrible at/i })).toHaveAttribute('href', '/vocabulary/vocabulary-1')
    expect(screen.getByText('Contributing lessons')).toBeInTheDocument()
  })

  it('sends search, status, sort and predictable pagination to the backend hook', () => {
    mocks.useVocabularyData.mockReturnValue({ data: { ...vocabularyPage, total: 26 }, loading: false, error: null, reload })
    render(<MemoryRouter><VocabularyPage /></MemoryRouter>)
    fireEvent.change(screen.getByPlaceholderText('Search word, phrase, or meaning'), { target: { value: 'terrible' } })
    expect(mocks.useVocabularyData).toHaveBeenLastCalledWith('all', 'terrible', 'recently_seen', 25, 0)
    fireEvent.click(screen.getByRole('button', { name: 'Known' }))
    expect(mocks.useVocabularyData).toHaveBeenLastCalledWith('known', 'terrible', 'recently_seen', 25, 0)
    fireEvent.change(screen.getByLabelText('Sort vocabulary'), { target: { value: 'alphabetical' } })
    expect(mocks.useVocabularyData).toHaveBeenLastCalledWith('known', 'terrible', 'alphabetical', 25, 0)
    fireEvent.click(screen.getByRole('button', { name: 'Next' }))
    expect(mocks.useVocabularyData).toHaveBeenLastCalledWith('known', 'terrible', 'alphabetical', 25, 25)
  })
})
