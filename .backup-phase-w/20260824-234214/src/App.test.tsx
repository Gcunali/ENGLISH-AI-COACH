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
vi.mock('./pages/AchievementsPage', () => ({ AchievementsPage: () => <div>Achievements route content</div> }))
vi.mock('./pages/ReviewPage', () => ({ ReviewPage: () => <div>Review route content</div> }))
vi.mock('./pages/ReviewSessionPage', () => ({ ReviewSessionPage: () => <div>Review session route content</div> }))
vi.mock('./pages/PlacementPage', () => ({ PlacementPage: () => <div>Placement route content</div> }))
vi.mock('./pages/PlacementResultPage', () => ({ PlacementResultPage: () => <div>Placement result route content</div> }))
vi.mock('./pages/VocabularyPage', () => ({ VocabularyPage: () => <div>Vocabulary route content</div> }))
vi.mock('./pages/VocabularyDetailsPage', () => ({ VocabularyDetailsPage: () => <div>Vocabulary detail route content</div> }))
vi.mock('./pages/SettingsPage', () => ({ SettingsPage: () => <div>Settings route content</div> }))
vi.mock('./pages/DiagnosticsPage', () => ({ DiagnosticsPage: () => <div>Diagnostics route content</div> }))
vi.mock('./pages/GuidedLessonsPage', () => ({ GuidedLessonsPage: () => <div>Guided lessons route content</div> }))
vi.mock('./pages/GuidedLessonDetailPage', () => ({ GuidedLessonDetailPage: () => <div>Guided lesson detail route content</div> }))
vi.mock('./pages/GuidedLessonSessionPage', () => ({ GuidedLessonSessionPage: () => <div>Guided lesson session route content</div> }))

import App from './App'

afterEach(cleanup)

describe('application routing', () => {
  it.each([
    ['/', 'Dashboard route content'],
    ['/history', 'History route content'],
    ['/history/lesson-1', 'Lesson detail route content'],
    ['/lesson/new', 'New lesson route content'],
    ['/progress', 'Progress route content'],
    ['/achievements', 'Achievements route content'],
    ['/review', 'Review route content'],
    ['/review/session/review-1', 'Review session route content'],
    ['/placement', 'Placement route content'],
    ['/placement/results/attempt-1', 'Placement result route content'],
    ['/vocabulary', 'Vocabulary route content'],
    ['/vocabulary/vocabulary-1', 'Vocabulary detail route content'],
    ['/settings', 'Settings route content'],
    ['/diagnostics', 'Diagnostics route content'],
    ['/guided-lessons', 'Guided lessons route content'],
    ['/guided-lessons/greetings-a1', 'Guided lesson detail route content'],
    ['/guided-lessons/session/session-1', 'Guided lesson session route content'],
  ])('renders %s', (path, expected) => {
    render(<MemoryRouter initialEntries={[path]}><App /></MemoryRouter>)
    expect(screen.getByText(expected)).toBeInTheDocument()
  })

  it('shows a friendly page for an invalid route', async () => {
    render(<MemoryRouter initialEntries={['/missing']}><App /></MemoryRouter>)
    expect(await screen.findByRole('heading', { name: 'That page is not available.' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Back to Home' })).toHaveAttribute('href', '/')
  })

  it('exposes landmarks, skip navigation and semantic active-route state', () => {
    render(<MemoryRouter initialEntries={['/history']}><App /></MemoryRouter>)
    expect(screen.getByRole('link', { name: 'Skip to main content' })).toHaveAttribute('href', '#main-content')
    expect(screen.getByRole('main')).toHaveAttribute('id', 'main-content')
    expect(screen.getAllByRole('link', { name: 'History' }).some((link) => link.getAttribute('aria-current') === 'page')).toBe(true)
  })
})
