import { Component, type ErrorInfo, type ReactNode } from 'react'
import { Link } from 'react-router-dom'
import { AlertCircle } from 'lucide-react'

export class RouteErrorBoundary extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false }

  static getDerivedStateFromError() { return { failed: true } }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('Route rendering failed', error, info)
  }

  render() {
    if (!this.state.failed) return this.props.children
    return <div className="page-shell page-shell-narrow"><section className="state-card" role="alert"><AlertCircle aria-hidden="true" className="state-icon text-[var(--danger)]" size={24} /><h1 className="page-title">Something went wrong on this page.</h1><p className="page-description">Your local data was not changed. Try displaying the page again or return to Home.</p><div className="state-actions"><button type="button" className="button-primary" onClick={() => this.setState({ failed: false })}>Try again</button><Link className="button-secondary" to="/">Back to Home</Link></div></section></div>
  }
}
