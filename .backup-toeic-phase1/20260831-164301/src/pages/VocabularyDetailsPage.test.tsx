// @vitest-environment jsdom

import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { vocabularyDetails } from '../test/learningFixtures'

const mocks = vi.hoisted(() => ({ useVocabularyItemData: vi.fn(), updateVocabularyStatus: vi.fn(), notify: vi.fn() }))
vi.mock('../hooks/useLearningData', () => ({ useVocabularyItemData: mocks.useVocabularyItemData }))
vi.mock('../services/native', () => ({ updateVocabularyStatus: mocks.updateVocabularyStatus }))
vi.mock('../utils/learningData', () => ({ notifyLearningDataChanged: mocks.notify }))
import { VocabularyDetailsPage } from './VocabularyDetailsPage'

const reload = vi.fn()
function renderPage() { return render(<MemoryRouter initialEntries={['/vocabulary/vocabulary-1']}><Routes><Route path="/vocabulary/:vocabularyId" element={<VocabularyDetailsPage />} /></Routes></MemoryRouter>) }

describe('VocabularyDetailsPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.useVocabularyItemData.mockReturnValue({ data: vocabularyDetails, loading: false, error: null, reload })
    mocks.updateVocabularyStatus.mockResolvedValue({ ...vocabularyDetails.item, status: 'learning' })
  })
  afterEach(cleanup)

  it('shows persisted meaning, example, status and lesson navigation', () => {
    renderPage()
    expect(screen.getByRole('heading', { name: 'terrible at' })).toBeInTheDocument()
    expect(screen.getByText('muito ruim em')).toBeInTheDocument()
    expect(screen.getByText(/terrible at math/)).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'View lesson' })).toHaveAttribute('href', '/history/lesson-1')
    expect(screen.getByLabelText('Learning status')).toHaveValue('new')
  })

  it('persists a manual status change and refreshes learning data', async () => {
    renderPage()
    fireEvent.change(screen.getByLabelText('Learning status'), { target: { value: 'learning' } })
    expect(screen.getByLabelText('Learning status')).toHaveValue('learning')
    await waitFor(() => expect(mocks.updateVocabularyStatus).toHaveBeenCalledWith('vocabulary-1', 'learning'))
    expect(mocks.notify).toHaveBeenCalled()
    expect(reload).toHaveBeenCalled()
  })
})
