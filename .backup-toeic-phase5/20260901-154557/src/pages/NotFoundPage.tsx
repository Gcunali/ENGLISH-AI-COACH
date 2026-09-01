import { Compass } from 'lucide-react'
import { Link } from 'react-router-dom'
import { PageHeader, PageShell } from '../components/ProductUI'

export function NotFoundPage() {
  return <PageShell width="narrow"><PageHeader eyebrow="Page not found" title="That page is not available." description="The address may be incomplete or the page may have moved." /><section className="state-card"><Compass aria-hidden="true" className="state-icon text-[var(--accent)]" size={28} /><p className="page-description">Your learning data is safe. Use the link below to continue.</p><Link to="/" className="button-primary">Back to Home</Link></section></PageShell>
}
