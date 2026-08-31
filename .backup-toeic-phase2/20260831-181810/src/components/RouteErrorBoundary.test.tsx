// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { cleanup, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { RouteErrorBoundary } from './RouteErrorBoundary'

function BrokenPage(): never { throw new Error('private technical detail') }

afterEach(() => { cleanup(); vi.restoreAllMocks() })

describe('RouteErrorBoundary', () => {
  it('keeps a friendly recovery UI and does not expose the stack or error detail', () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined)
    render(<MemoryRouter><RouteErrorBoundary><BrokenPage /></RouteErrorBoundary></MemoryRouter>)
    expect(screen.getByRole('heading', { name: 'Something went wrong on this page.' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Back to Home' })).toHaveAttribute('href', '/')
    expect(screen.queryByText('private technical detail')).not.toBeInTheDocument()
  })
})
