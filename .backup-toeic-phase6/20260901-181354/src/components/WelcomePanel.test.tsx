// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const native = vi.hoisted(() => ({ getWelcomeState: vi.fn(), setWelcomeSeen: vi.fn() }))
vi.mock('../services/native', () => native)
import { WelcomePanel } from './WelcomePanel'

beforeEach(() => {
  native.getWelcomeState.mockResolvedValue({ shouldShow: true, hasSeen: false, existingUser: false })
  native.setWelcomeSeen.mockResolvedValue({ shouldShow: false, hasSeen: true, existingUser: false })
})
afterEach(() => { cleanup(); vi.clearAllMocks() })

describe('WelcomePanel', () => {
  it('guides a new user without blocking navigation and persists dismissal', async () => {
    render(<MemoryRouter><WelcomePanel /></MemoryRouter>)
    expect(await screen.findByRole('heading', { name: 'Welcome to your English learning space.' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Open Course' })).toHaveAttribute('href', '/course')
    expect(screen.getByRole('link', { name: 'Optional Placement Test' })).toHaveAttribute('href', '/placement')
    expect(screen.getByRole('link', { name: 'Free Conversation' })).toHaveAttribute('href', '/lesson/new')
    expect(screen.getByRole('link', { name: 'Check Readiness' })).toHaveAttribute('href', '/diagnostics')
    fireEvent.click(screen.getByRole('button', { name: 'Dismiss welcome' }))
    await waitFor(() => expect(native.setWelcomeSeen).toHaveBeenCalledWith(true))
    expect(screen.queryByRole('heading', { name: 'Welcome to your English learning space.' })).not.toBeInTheDocument()
  })

  it('does not show onboarding for an existing user', async () => {
    native.getWelcomeState.mockResolvedValue({ shouldShow: false, hasSeen: true, existingUser: true })
    render(<MemoryRouter><WelcomePanel /></MemoryRouter>)
    await waitFor(() => expect(native.getWelcomeState).toHaveBeenCalledOnce())
    expect(screen.queryByText(/Welcome to your English/)).not.toBeInTheDocument()
  })
})
