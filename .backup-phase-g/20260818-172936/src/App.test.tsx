// @vitest-environment jsdom

import '@testing-library/jest-dom/vitest'
import { cleanup, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { afterEach, describe, expect, it, vi } from 'vitest'

vi.mock('./pages/DashboardPage', () => ({ DashboardPage: () => <div>Dashboard route content</div> }))
vi.mock('./pages/HistoryPage', () => ({ HistoryPage: () => <div>History route content</div> }))
vi.mock('./pages/LessonDetailsPage', () => ({ LessonDetailsPage: () => <div>Lesson detail route content</div> }))
vi.mock('./pages/ProgressPage', () => ({ ProgressPage: () => <div>Progress route content</div> }))
vi.mock('./pages/PlaceholderPage', () => ({ PlaceholderPage: ({ title }: { title: string }) => <div>{title} placeholder</div> }))

import App from './App'

afterEach(cleanup)

describe('application routing', () => {
  it.each([
    ['/', 'Dashboard route content'],
    ['/history', 'History route content'],
    ['/history/lesson-1', 'Lesson detail route content'],
    ['/progress', 'Progress route content'],
    ['/settings', 'Settings placeholder'],
  ])('renders %s', (path, expected) => {
    render(<MemoryRouter initialEntries={[path]}><App /></MemoryRouter>)
    expect(screen.getByText(expected)).toBeInTheDocument()
  })

  it('redirects an invalid route to Home', async () => {
    render(<MemoryRouter initialEntries={['/missing']}><App /></MemoryRouter>)
    expect(await screen.findByText('Dashboard route content')).toBeInTheDocument()
  })
})
