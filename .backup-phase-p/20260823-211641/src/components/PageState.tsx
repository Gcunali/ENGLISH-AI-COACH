export function LoadingState({ label = 'Loading local data…' }: { label?: string }) {
  return <div role="status" className="glass rounded-2xl p-6 text-sm muted">{label}</div>
}

export function ErrorState({ message, onRetry }: { message: string; onRetry: () => void }) {
  return <div role="alert" className="glass rounded-2xl p-6">
    <p className="mt-0 text-red-200">Local data could not be loaded.</p>
    <p className="muted text-sm">{message}</p>
    <button onClick={onRetry} className="rounded-full border border-white/15 bg-white/5 px-4 py-2 text-sm text-white">Try again</button>
  </div>
}

export function EmptyState({ title, message }: { title: string; message: string }) {
  return <div className="glass rounded-2xl p-8 text-center">
    <h2 className="mt-0 text-lg">{title}</h2><p className="muted mb-0 text-sm">{message}</p>
  </div>
}
