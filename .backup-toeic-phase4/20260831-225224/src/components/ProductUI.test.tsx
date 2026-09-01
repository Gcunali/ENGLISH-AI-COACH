// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { EmptyState } from './PageState'
import { InlineNotice, PageHeader, ToggleRow } from './ProductUI'

afterEach(cleanup)

describe('shared product UI', () => {
  it('provides a single page heading and labeled switch semantics', () => {
    const onChange = vi.fn()
    render(<><PageHeader eyebrow="Section" title="Page title" description="Description"/><ToggleRow label="Private setting" description="Stored locally." checked={false} onChange={onChange}/></>)
    expect(screen.getByRole('heading', { level: 1, name: 'Page title' })).toBeInTheDocument()
    const toggle = screen.getByRole('switch', { name: 'Private setting' })
    expect(toggle).toHaveAttribute('aria-checked', 'false')
    fireEvent.click(toggle)
    expect(onChange).toHaveBeenCalledOnce()
  })

  it('renders actionable empty states and text-backed status semantics', () => {
    render(<><EmptyState title="Nothing here" message="Complete a lesson first." action={<button>Start</button>}/><InlineNotice tone="warning">Component unavailable</InlineNotice></>)
    expect(screen.getByRole('button', { name: 'Start' })).toBeInTheDocument()
    expect(screen.getByRole('note')).toHaveTextContent('Component unavailable')
  })
})
