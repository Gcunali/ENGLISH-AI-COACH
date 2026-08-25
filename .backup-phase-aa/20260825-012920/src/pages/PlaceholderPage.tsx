import { EmptyState } from '../components/PageState'

export function PlaceholderPage({ title }: { title: string }) {
  return <><header className="mb-7"><p className="muted text-xs uppercase tracking-[.18em] mb-2">Future workspace</p><h1 className="m-0 text-2xl md:text-3xl">{title}</h1></header><EmptyState title={`${title} is not implemented yet`} message="This area remains visible in the navigation but is intentionally outside Phase F." /></>
}
