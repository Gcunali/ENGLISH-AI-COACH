import { AlertCircle, CheckCircle2, Info, TriangleAlert } from 'lucide-react'
import type { ReactNode } from 'react'

export function PageShell({ children, width = 'standard' }: { children: ReactNode; width?: 'narrow' | 'standard' | 'wide' }) {
  return <div className={`page-shell page-shell-${width}`}>{children}</div>
}

export function PageHeader({ eyebrow, title, description, actions, back }: { eyebrow?: string; title: string; description?: string; actions?: ReactNode; back?: ReactNode }) {
  return <header className="page-header">
    {back && <div className="page-header-back">{back}</div>}
    <div className="page-header-row"><div className="min-w-0">{eyebrow && <p className="eyebrow">{eyebrow}</p>}<h1 className="page-title">{title}</h1>{description && <p className="page-description">{description}</p>}</div>{actions && <div className="page-actions">{actions}</div>}</div>
  </header>
}

export function SectionHeader({ title, description, actions, id }: { title: string; description?: string; actions?: ReactNode; id?: string }) {
  return <div className="section-header"><div className="min-w-0"><h2 id={id} className="section-title">{title}</h2>{description && <p className="section-description">{description}</p>}</div>{actions && <div className="section-actions">{actions}</div>}</div>
}

const NOTICE_ICONS = { success: CheckCircle2, warning: TriangleAlert, error: AlertCircle, info: Info } as const

export function InlineNotice({ tone = 'info', title, children, live = false }: { tone?: keyof typeof NOTICE_ICONS; title?: string; children: ReactNode; live?: boolean }) {
  const Icon = NOTICE_ICONS[tone]
  return <div className={`notice notice-${tone}`} role={tone === 'error' ? 'alert' : live ? 'status' : 'note'} aria-live={live ? (tone === 'error' ? 'assertive' : 'polite') : undefined}>
    <Icon aria-hidden="true" size={18} className="notice-icon" />
    <div className="min-w-0">{title && <strong className="notice-title">{title}</strong>}<div className="notice-body">{children}</div></div>
  </div>
}

export function StatusBadge({ tone = 'neutral', children }: { tone?: 'success' | 'warning' | 'error' | 'info' | 'neutral'; children: ReactNode }) {
  return <span className={`status-badge status-${tone}`}><span aria-hidden="true" className="status-indicator" />{children}</span>
}

export function MetricCard({ label, value, detail }: { label: string; value: ReactNode; detail?: ReactNode }) {
  return <div className="metric-card"><div className="metric-label">{label}</div><div className="metric-value">{value}</div>{detail && <div className="metric-detail">{detail}</div>}</div>
}

export function ToggleRow({ label, description, checked, busy = false, onChange, icon }: { label: string; description: string; checked: boolean | null; busy?: boolean; onChange: () => void; icon?: ReactNode }) {
  return <div className="flex flex-col items-start justify-between gap-5 sm:flex-row sm:items-center"><div className="flex min-w-0 flex-1 gap-3">{icon && <span aria-hidden="true" className="mt-1 shrink-0 text-[var(--accent)]">{icon}</span>}<div><h3 className="m-0 text-sm font-semibold">{label}</h3><p className="section-description mt-1">{description}</p></div></div><button type="button" role="switch" aria-label={label} aria-checked={checked ?? false} aria-busy={busy} disabled={checked === null || busy} onClick={onChange} className={`toggle-control ${checked ? 'toggle-control-on' : ''}`}><span aria-hidden="true" className="toggle-thumb"/><span className="sr-only">{checked ? 'On' : 'Off'}</span></button></div>
}
