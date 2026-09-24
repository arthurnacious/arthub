import { useQuery } from '@tanstack/react-query'
import { ContributionGraph } from '../charts/ContributionGraph'
import { ErrorState } from '../ErrorState'
import { Skeleton } from '../Skeleton'
import { fetchActivityHeatmap } from '../../lib/api/activity'
import { formatLocalDate } from '../../lib/local-date'
import type { ContributionDay } from '../charts/ContributionGraph'

// 53 weeks (371 days) so a full 12 months always show, ending on the
// current day.
const WINDOW_DAYS = 371

// The backend only returns a row for a day that actually had activity
// (see ActivityController::heatmap) - this fills in every other day in
// the window as a zero, so the graph always renders a full, stable grid
// rather than shrinking to whatever span the sparse data happens to
// cover (ContributionGraph's own week-alignment infers its range from
// the data it's given, so an incomplete range would just render a
// shorter graph, not padding).
function fillWindow(counts: { date: string; count: number }[]): ContributionDay[] {
  const byDate = new Map(counts.map((d) => [d.date, d.count]))
  const today = new Date()
  const days: ContributionDay[] = []

  for (let i = WINDOW_DAYS - 1; i >= 0; i--) {
    const date = new Date(today.getFullYear(), today.getMonth(), today.getDate() - i)
    const iso = formatLocalDate(date)
    days.push({ date: iso, count: byDate.get(iso) ?? 0 })
  }

  return days
}

// Its own query, isolated from every other dashboard panel.
export function ActivityGraphPanel() {
  const { data, isPending, isError, refetch } = useQuery({
    queryKey: ['activity-heatmap', WINDOW_DAYS],
    queryFn: () => fetchActivityHeatmap(WINDOW_DAYS),
  })

  if (isPending) {
    return (
      <div className="flex h-24 items-end gap-1">
        {Array.from({ length: 24 }).map((_, i) => (
          <Skeleton key={i} className={`h-full w-3 ${i % 4 === 0 ? 'opacity-30' : ''}`} />
        ))}
      </div>
    )
  }

  if (isError) {
    return (
      <ErrorState
        title="COULDN'T LOAD ACTIVITY GRAPH"
        message="Something went wrong fetching the activity heatmap."
        onRetry={() => refetch()}
      />
    )
  }

  return <ContributionGraph data={fillWindow(data ?? [])} />
}
