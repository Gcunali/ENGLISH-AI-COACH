import { AlertCircle, Inbox, LoaderCircle } from 'lucide-react'
import type { ReactNode } from 'react'

export function LoadingState({ label = 'Loading local data…' }: { label?: string }) {
  return <div role="status" aria-live="polite" aria-busy="true" className="state-card state-card-compact"><LoaderCircle aria-hidden="true" className="state-icon animate-spin text-[var(--accent)]" size={21} /><span>{label}</span></div>
}

export function ErrorState({ message, onRetry, title = 'Local data could not be loaded.' }: { message: string; onRetry: () => void; title?: string }) {
  return <div role="alert" className="state-card"><AlertCircle aria-hidden="true" className="state-icon text-[var(--danger)]" size={23} /><h2 className="section-title">{title}</h2><p className="page-description break-words">{message}</p><button type="button" onClick={onRetry} className="button-secondary">Try again</button></div>
}

export function EmptyState({ title, message, action }: { title: string; message: string; action?: ReactNode }) {
  return <div className="state-card text-center"><Inbox aria-hidden="true" className="state-icon mx-auto text-[var(--muted)]" size={25} /><h2 className="section-title">{title}</h2><p className="page-description mx-auto max-w-xl">{message}</p>{action && <div className="state-actions justify-center">{action}</div>}</div>
}
