// @vitest-environment jsdom

import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { LessonModeDefinition } from '../types'

const mocks = vi.hoisted(() => ({ getLessonModes: vi.fn(), startLesson: vi.fn() }))
vi.mock('../services/native', () => ({ getLessonModes: mocks.getLessonModes }))
vi.mock('../hooks/useLocalVoiceEngine', () => ({ useLocalVoiceEngine: () => ({ startLesson: mocks.startLesson, engineState: 'stopped', endLesson: vi.fn() }) }))
import { NewLessonPage } from './NewLessonPage'

const ids = ['free_conversation', 'everyday_english', 'travel_english', 'job_interview', 'university_academic', 'debate_opinions', 'custom'] as const
const titles = ['Free Conversation', 'Everyday English', 'Travel English', 'Job Interview', 'University / Academic English', 'Debate & Opinions', 'Custom Lesson']
const modes: LessonModeDefinition[] = ids.map((id, index) => ({ id, version: 1, title: titles[index], description: `${titles[index]} description`, defaultDifficulty: 'standard', supportedDifficulties: ['easy', 'standard', 'challenging'], availableFocusAreas: id === 'custom' ? ['vocabulary', 'naturalness'] : [], allowsTopic: id === 'custom', allowsObjective: id === 'custom', allowsScenario: id === 'custom', allowsCustomTitle: id === 'custom' }))

function page() { return render(<MemoryRouter initialEntries={['/lesson/new']}><Routes><Route path="/lesson/new" element={<NewLessonPage />} /><Route path="/" element={<div>Dashboard destination</div>} /></Routes></MemoryRouter>) }
beforeEach(() => { mocks.getLessonModes.mockResolvedValue(modes); mocks.startLesson.mockResolvedValue(true) })
afterEach(() => { cleanup(); vi.clearAllMocks() })

describe('NewLessonPage', () => {
  it('loads the backend registry and shows all seven cards', async () => {
    page()
    expect(screen.getByText('Loading lesson modes from the local registry…')).toBeInTheDocument()
    for (const title of titles) expect(await screen.findByRole('button', { name: new RegExp(title) })).toBeInTheDocument()
  })

  it('selects and goes back from a preset configuration', async () => {
    page(); fireEvent.click(await screen.findByRole('button', { name: /Travel English/ }))
    expect(screen.getByText('Lesson preview')).toBeInTheDocument()
    expect(screen.getAllByText('Standard')).toHaveLength(2)
    fireEvent.click(screen.getByRole('button', { name: /Choose another mode/ }))
    expect(await screen.findByRole('button', { name: /Free Conversation/ })).toBeInTheDocument()
  })

  it('requires custom topic, previews fields and starts through the shared backend request', async () => {
    page(); fireEvent.click(await screen.findByRole('button', { name: /Custom Lesson/ }))
    const start = screen.getByRole('button', { name: 'Start Lesson' })
    expect(start).toBeDisabled()
    fireEvent.change(screen.getByLabelText('Topic'), { target: { value: 'Ordering food at a restaurant' } })
    fireEvent.change(screen.getByLabelText('Objective (optional)'), { target: { value: 'Speak naturally with a waiter' } })
    fireEvent.click(screen.getByRole('button', { name: 'Vocabulary' }))
    fireEvent.click(screen.getByRole('button', { name: 'Naturalness' }))
    expect(screen.getAllByText('Ordering food at a restaurant').length).toBeGreaterThan(0)
    fireEvent.click(start)
    await waitFor(() => expect(mocks.startLesson).toHaveBeenCalledWith({ modeId: 'custom', difficulty: 'standard', topic: 'Ordering food at a restaurant', objective: 'Speak naturally with a waiter', focusAreas: ['vocabulary', 'naturalness'] }))
    expect(await screen.findByText('Dashboard destination')).toBeInTheDocument()
  })
})
