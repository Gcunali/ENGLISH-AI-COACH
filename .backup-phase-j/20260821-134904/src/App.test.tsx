// @vitest-environment jsdom

import '@testing-library/jest-dom/vitest'
import { cleanup, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { afterEach, describe, expect, it, vi } from 'vitest'

vi.mock('./pages/DashboardPage', () => ({ DashboardPage: () => <div>Dashboard route content</div> }))
vi.mock('./pages/HistoryPage', () => ({ HistoryPage: () => <div>History route content</div> }))
vi.mock('./pages/LessonDetailsPage', () => ({ LessonDetailsPage: () => <div>Lesson detail route content</div> }))
vi.mock('./pages/NewLessonPage', () => ({ NewLessonPage: () => <div>New lesson route content</div> }))
vi.mock('./pages/ProgressPage', () => ({ ProgressPage: () => <div>Progress route content</div> }))
vi.mock('./pages/VocabularyPage', () => ({ VocabularyPage: () => <div>Vocabulary route content</div> }))
vi.mock('./pages/VocabularyDetailsPage', () => ({ VocabularyDetailsPage: () => <div>Vocabulary detail route content</div> }))
vi.mock('./pages/SettingsPage', () => ({ SettingsPage: () => <div>Settings route content</div> }))

import App from './App'

afterEach(cleanup)

describe('application routing', () => {
  it.each([
    ['/', 'Dashboard route content'],
    ['/history', 'History route content'],
    ['/history/lesson-1', 'Lesson detail route content'],
    ['/lesson/new', 'New lesson route content'],
    ['/progress', 'Progress route content'],
    ['/vocabulary', 'Vocabulary route content'],
    ['/vocabulary/vocabulary-1', 'Vocabulary detail route content'],
    ['/settings', 'Settings route content'],
  ])('renders %s', (path, expected) => {
    render(<MemoryRouter initialEntries={[path]}><App /></MemoryRouter>)
    expect(screen.getByText(expected)).toBeInTheDocument()
  })

  it('redirects an invalid route to Home', async () => {
    render(<MemoryRouter initialEntries={['/missing']}><App /></MemoryRouter>)
    expect(await screen.findByText('Dashboard route content')).toBeInTheDocument()
  })
})
