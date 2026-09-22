export interface BarDatum {
  label: string
  value: number
  // Renders as a flat gray placeholder instead of scaling by value - for
  // categories that exist (e.g. future months) but have no real data yet.
  muted?: boolean
  // Shown once, above the first bar of each new group (e.g. a year label
  // above the first month of that year, for a rolling multi-year window).
  group?: string
}

const GROUP_LABEL_HEIGHT = 14
const CHART_HEIGHT = 96
const LABEL_HEIGHT = 20
const BAR_WIDTH = 24
const BAR_GAP = 16
const MUTED_HEIGHT = 4

interface GroupLabel {
  index: number
  label: string
}

// One label per distinct `group` value, at the index of its first bar.
function groupLabels(data: BarDatum[]): GroupLabel[] {
  const labels: GroupLabel[] = []
  let last: string | undefined

  data.forEach((d, index) => {
    if (d.group !== undefined && d.group !== last) {
      labels.push({ index, label: d.group })
      last = d.group
    }
  })

  return labels
}

// Simple vertical bar chart. Bar height is proportional to the largest
// non-muted value in the dataset (not a fixed scale), so it always uses
// the full available height.
export function BarChart({ data }: { data: BarDatum[] }) {
  if (data.length === 0) return null

  const max = Math.max(1, ...data.filter((d) => !d.muted).map((d) => d.value))
  const hasGroups = data.some((d) => d.group !== undefined)
  const topOffset = hasGroups ? GROUP_LABEL_HEIGHT : 0
  const viewWidth = data.length * (BAR_WIDTH + BAR_GAP)
  const viewHeight = topOffset + CHART_HEIGHT + LABEL_HEIGHT

  return (
    <svg
      role="img"
      aria-label={`Bar chart: ${data.map((d) => `${d.label}${d.group ? ` ${d.group}` : ''} ${d.muted ? 'no data' : d.value}`).join(', ')}`}
      viewBox={`0 0 ${viewWidth} ${viewHeight}`}
      style={{ width: '100%', height: 'auto', aspectRatio: `${viewWidth} / ${viewHeight}` }}
    >
      {hasGroups && (
        <line
          data-testid="group-row-border"
          x1={0}
          y1={GROUP_LABEL_HEIGHT - 1}
          x2={viewWidth}
          y2={GROUP_LABEL_HEIGHT - 1}
          className="stroke-[var(--border)]"
        />
      )}
      {hasGroups &&
        groupLabels(data).map(({ index, label }, gi) => (
          <g key={label}>
            <text
              data-testid="group-label"
              x={index * (BAR_WIDTH + BAR_GAP)}
              y={GROUP_LABEL_HEIGHT - 4}
              className="fill-[var(--text-dim)] text-[10px] font-bold"
            >
              {label}
            </text>
            {gi > 0 && (
              <line
                data-testid="group-divider"
                x1={index * (BAR_WIDTH + BAR_GAP) - BAR_GAP / 2}
                y1={0}
                x2={index * (BAR_WIDTH + BAR_GAP) - BAR_GAP / 2}
                y2={viewHeight}
                className="stroke-[var(--border)]"
              />
            )}
          </g>
        ))}
      {data.map((d, i) => {
        const barHeight = d.muted ? MUTED_HEIGHT : (d.value / max) * CHART_HEIGHT
        const x = i * (BAR_WIDTH + BAR_GAP)

        return (
          <g key={`${d.group ?? ''}-${d.label}`} data-testid="bar">
            <rect
              data-testid="bar-rect"
              x={x}
              y={topOffset + CHART_HEIGHT - barHeight}
              width={BAR_WIDTH}
              height={barHeight}
              rx={2}
              className={
                d.muted
                  ? 'fill-[var(--border)]'
                  : 'fill-[var(--accent)] opacity-80'
              }
            />
            <text
              x={x + BAR_WIDTH / 2}
              y={topOffset + CHART_HEIGHT + 14}
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
