interface Point {
  label: string
  value: number
}

export function ProgressChart({ title, points }: { title: string; points: Point[] }) {
  const width = 640
  const height = 220
  const padding = 30
  const usableWidth = width - padding * 2
  const usableHeight = height - padding * 2
  const plotted = points.map((point, index) => ({
    ...point,
    x: points.length === 1 ? width / 2 : padding + (index * usableWidth) / (points.length - 1),
    y: padding + ((100 - point.value) * usableHeight) / 100,
  }))
  const line = plotted.map((point) => `${point.x},${point.y}`).join(' ')

  return <section className="glass rounded-2xl p-5" aria-label={title}>
    <h2 className="mt-0 text-base">{title}</h2>
    <svg viewBox={`0 0 ${width} ${height}`} role="img" aria-label={`${title}. ${points.length} real data point${points.length === 1 ? '' : 's'}.`} className="h-auto w-full overflow-visible">
      {[0, 25, 50, 75, 100].map((value) => {
        const y = padding + ((100 - value) * usableHeight) / 100
        return <g key={value}><line x1={padding} y1={y} x2={width - padding} y2={y} stroke="#e3edf8" /><text x={2} y={y + 4} fill="var(--muted)" fontSize="11">{value}</text></g>
      })}
      {plotted.length > 1 && <polyline points={line} fill="none" stroke="var(--accent)" strokeWidth="3" />}
      {plotted.map((point) => <g key={`${point.label}-${point.x}`} tabIndex={0} aria-label={`${point.label}: ${point.value}`}>
        <circle cx={point.x} cy={point.y} r="6" fill="var(--accent)" stroke="#ffffff" strokeWidth="2" />
      </g>)}
    </svg>
    <ol className="mt-3 grid gap-2 p-0 text-xs md:grid-cols-2" aria-label={`${title} values`}>
      {points.map((point) => <li key={point.label} className="flex justify-between rounded-lg bg-white/[.035] px-3 py-2"><span className="muted">{point.label}</span><strong>{point.value}</strong></li>)}
    </ol>
  </section>
}
