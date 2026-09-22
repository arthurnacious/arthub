export interface ContributionDay {
  date: string // ISO YYYY-MM-DD
  count: number
}

const CELL = 11
const GAP = 3

// Local-time date helpers (not UTC / toISOString round-tripping), so this
// never shifts a day depending on the viewer's timezone offset.
function parseLocalDate(iso: string): Date {
  const [y, m, d] = iso.split('-').map(Number)
  return new Date(y, m - 1, d)
}

function formatLocalDate(date: Date): string {
  const y = date.getFullYear()
  const m = String(date.getMonth() + 1).padStart(2, '0')
  const d = String(date.getDate()).padStart(2, '0')
  return `${y}-${m}-${d}`
}

function levelFor(count: number): 0 | 1 | 2 | 3 | 4 {
  if (count <= 0) return 0
  if (count === 1) return 1
  if (count <= 3) return 2
  if (count <= 6) return 3
  return 4
}

const LEVEL_CLASSES: Record<0 | 1 | 2 | 3 | 4, string> = {
  0: 'fill-[var(--border)]',
  1: 'fill-[var(--accent)] opacity-30',
  2: 'fill-[var(--accent)] opacity-55',
  3: 'fill-[var(--accent)] opacity-80',
  4: 'fill-[var(--accent)] opacity-100',
}

function buildWeeks(data: ContributionDay[]): ContributionDay[][] {
  if (data.length === 0) return []

  const byDate = new Map(data.map((d) => [d.date, d]))
  const dates = data.map((d) => parseLocalDate(d.date).getTime())
  const start = parseLocalDate(formatLocalDate(new Date(Math.min(...dates))))
  const end = parseLocalDate(formatLocalDate(new Date(Math.max(...dates))))

  const alignedStart = new Date(start)
  alignedStart.setDate(alignedStart.getDate() - alignedStart.getDay())

  const alignedEnd = new Date(end)
  alignedEnd.setDate(alignedEnd.getDate() + (6 - alignedEnd.getDay()))

  const weeks: ContributionDay[][] = []
  const cursor = new Date(alignedStart)
  let week: ContributionDay[] = []

  while (cursor <= alignedEnd) {
    const iso = formatLocalDate(cursor)
    week.push(byDate.get(iso) ?? { date: iso, count: 0 })
    if (week.length === 7) {
      weeks.push(week)
      week = []
    }
    cursor.setDate(cursor.getDate() + 1)
  }

  return weeks
}

// GitHub-style contribution heatmap: one column per week, one row per day
// (Sun-Sat), cell opacity scaled to that day's count.
export function ContributionGraph({ data }: { data: ContributionDay[] }) {
  const weeks = buildWeeks(data)
  if (weeks.length === 0) return null

  const total = data.reduce((sum, d) => sum + d.count, 0)
  const width = weeks.length * (CELL + GAP)
  const height = 7 * (CELL + GAP)

  return (
    <svg
      role="img"
      aria-label={`Contribution graph: ${total} events over ${data.length} days`}
      width={width}
      height={height}
      className="overflow-visible"
    >
      {weeks.map((week, wi) =>
        week.map((day, di) => {
          const level = levelFor(day.count)
          return (
            <rect
              key={day.date}
              data-testid="contribution-cell"
              data-date={day.date}
              data-level={level}
              x={wi * (CELL + GAP)}
              y={di * (CELL + GAP)}
              width={CELL}
              height={CELL}
              rx={2}
              className={LEVEL_CLASSES[level]}
            >
              <title>{`${day.date}: ${day.count}`}</title>
            </rect>
          )
        }),
      )}
    </svg>
  )
}
