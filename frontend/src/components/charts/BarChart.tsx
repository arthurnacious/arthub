export interface BarDatum {
  label: string
  value: number
}

const CHART_HEIGHT = 96
const LABEL_HEIGHT = 20
const BAR_WIDTH = 24
const BAR_GAP = 16

// Simple vertical bar chart. Bar height is proportional to the largest
// value in the dataset (not a fixed scale), so it always uses the full
// available height.
export function BarChart({ data }: { data: BarDatum[] }) {
  if (data.length === 0) return null

  const max = Math.max(1, ...data.map((d) => d.value))
  const width = data.length * (BAR_WIDTH + BAR_GAP)

  return (
    <svg
      role="img"
      aria-label={`Bar chart: ${data.map((d) => `${d.label} ${d.value}`).join(', ')}`}
      width={width}
      height={CHART_HEIGHT + LABEL_HEIGHT}
    >
      {data.map((d, i) => {
        const barHeight = (d.value / max) * CHART_HEIGHT
        const x = i * (BAR_WIDTH + BAR_GAP)

        return (
          <g key={d.label} data-testid="bar">
            <rect
              data-testid="bar-rect"
              x={x}
              y={CHART_HEIGHT - barHeight}
              width={BAR_WIDTH}
              height={barHeight}
              rx={2}
              className="fill-[var(--accent)] opacity-80"
            />
            <text
              x={x + BAR_WIDTH / 2}
              y={CHART_HEIGHT + 14}
              textAnchor="middle"
              className="fill-[var(--text-dim)] text-[10px]"
            >
              {d.label}
            </text>
          </g>
        )
      })}
    </svg>
  )
}
