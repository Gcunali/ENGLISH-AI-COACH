// @vitest-environment jsdom
import '@testing-library/jest-dom/vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { useState } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { ConfirmationDialog } from './ConfirmationDialog'

function Harness({ confirm = vi.fn() }: { confirm?: () => void }) {
  const [open, setOpen] = useState(false)
  return <><button onClick={() => setOpen(true)}>Open dialog</button><ConfirmationDialog open={open} title="Delete saved item?" description="This cannot be undone." confirmLabel="Delete" danger onConfirm={confirm} onClose={() => setOpen(false)} /></>
}

afterEach(cleanup)

describe('ConfirmationDialog', () => {
  it('moves focus inside, closes with Escape, and returns focus to its trigger', async () => {
    render(<Harness />)
    const trigger = screen.getByRole('button', { name: 'Open dialog' })
    trigger.focus()
    fireEvent.click(trigger)
    const cancel = screen.getByRole('button', { name: 'Cancel' })
    await waitFor(() => expect(cancel).toHaveFocus())
    fireEvent.keyDown(document, { key: 'Escape' })
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    await waitFor(() => expect(trigger).toHaveFocus())
  })

  it('provides an accessible dialog name and explicit destructive action', () => {
    const confirm = vi.fn()
    render(<Harness confirm={confirm} />)
    fireEvent.click(screen.getByRole('button', { name: 'Open dialog' }))
    const dialog = screen.getByRole('dialog', { name: 'Delete saved item?' })
    expect(dialog).toHaveAttribute('aria-modal', 'true')
    fireEvent.click(screen.getByRole('button', { name: 'Delete' }))
    expect(confirm).toHaveBeenCalledOnce()
  })
})
