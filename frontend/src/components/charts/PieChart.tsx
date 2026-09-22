import type { StatusTone } from '../StatusBadge'

export interface PieDatum {
  label: string
  value: number
  tone?: StatusTone
}

const TONE_STROKE: Record<StatusTone, string> = {
  success: 'stroke-emerald-400',
  warning: 'stroke-amber-400',
  danger: 'stroke-red-400',
  info: 'stroke-sky-400',
  neutral: 'stroke-zinc-400',
}

const SIZE = 120
const STROKE = 18
const RADIUS = (SIZE - STROKE) / 2
const CIRCUMFERENCE = 2 * Math.PI * RADIUS

// Donut chart built from stacked circle strokes (stroke-dasharray/-offset),
// rather than hand-rolled arc paths. Slices are ordered by `data`; a
// legend with each slice's share follows.
export function PieChart({ data }: { data: PieDatum[] }) {
  const total = data.reduce((sum, d) => sum + d.value, 0)
  let cumulative = 0

  return (
    <div className="flex items-center gap-4">
      <svg
        role="img"
        aria-label={`Pie chart: ${data.map((d) => `${d.label} ${d.value}`).join(', ')}`}
        width={SIZE}
        height={SIZE}
        viewBox={`0 0 ${SIZE} ${SIZE}`}
        className="-rotate-90"
      >
        {total === 0 ? (
          <circle
            cx={SIZE / 2}
            cy={SIZE / 2}
            r={RADIUS}
            fill="none"
            strokeWidth={STROKE}
            className="stroke-[var(--border)]"
          />
        ) : (
          data.map((d) => {
            const fraction = d.value / total
            const length = fraction * CIRCUMFERENCE
            const offset = cumulative
            cumulative += length

            return (
              <circle
                key={d.label}
                data-testid="pie-slice"
                cx={SIZE / 2}
                cy={SIZE / 2}
                r={RADIUS}
                fill="none"
                strokeWidth={STROKE}
                strokeDasharray={`${length} ${CIRCUMFERENCE - length}`}
                strokeDashoffset={-offset}
                className={TONE_STROKE[d.tone ?? 'neutral']}
              />
            )
          })
        )}
      </svg>
      <ul className="flex flex-col gap-1 text-xs">
        {data.map((d) => (
          <li key={d.label} className="flex items-center justify-between gap-3">
            <span className="text-[var(--text-dim)]">{d.label}</span>
            <span className="text-[var(--text)]">
              {total === 0 ? '0%' : `${Math.round((d.value / total) * 100)}%`}
            </span>
          </li>
        ))}
      </ul>
    </div>
  )
}
