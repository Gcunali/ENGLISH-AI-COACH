import { useCallback, useEffect, useId, useRef } from 'react'

export function ConfirmationDialog({ open, title, description, confirmLabel, cancelLabel = 'Cancel', danger = false, busy = false, onConfirm, onClose }: { open: boolean; title: string; description: React.ReactNode; confirmLabel: string; cancelLabel?: string; danger?: boolean; busy?: boolean; onConfirm: () => void; onClose: () => void }) {
  const titleId = useId()
  const descriptionId = useId()
  const panelRef = useRef<HTMLDivElement>(null)
  const cancelRef = useRef<HTMLButtonElement>(null)
  const previousFocus = useRef<HTMLElement | null>(null)
  const onCloseRef = useRef(onClose)
  const busyRef = useRef(busy)
  useEffect(() => { onCloseRef.current = onClose }, [onClose])
  useEffect(() => { busyRef.current = busy }, [busy])
  const requestClose = useCallback(() => {
    const target = previousFocus.current
    onCloseRef.current()
    window.setTimeout(() => target?.focus(), 0)
  }, [])

  useEffect(() => {
    if (!open) return undefined
    previousFocus.current = document.activeElement as HTMLElement | null
    window.requestAnimationFrame(() => cancelRef.current?.focus())
    const handleKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !busyRef.current) { event.preventDefault(); requestClose(); return }
      if (event.key !== 'Tab') return
      const focusable = panelRef.current?.querySelectorAll<HTMLElement>('button:not([disabled]), a[href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])')
      if (!focusable?.length) return
      const first = focusable[0]
      const last = focusable[focusable.length - 1]
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus() }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus() }
    }
    document.addEventListener('keydown', handleKey)
    return () => {
      document.removeEventListener('keydown', handleKey)
      const target = previousFocus.current
      window.setTimeout(() => target?.focus(), 0)
    }
  }, [open, requestClose])

  if (!open) return null
  return <div className="dialog-backdrop" role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget && !busy) requestClose() }}>
    <div ref={panelRef} role="dialog" aria-modal="true" aria-labelledby={titleId} aria-describedby={descriptionId} aria-busy={busy} className="dialog-panel">
      <h2 id={titleId} className="section-title">{title}</h2>
      <div id={descriptionId} className="page-description mt-3">{description}</div>
      <div className="dialog-actions"><button ref={cancelRef} type="button" disabled={busy} onClick={requestClose} className="button-secondary">{cancelLabel}</button><button type="button" disabled={busy} onClick={onConfirm} className={danger ? 'button-danger' : 'button-primary'}>{busy ? 'Working...' : confirmLabel}</button></div>
    </div>
  </div>
}
