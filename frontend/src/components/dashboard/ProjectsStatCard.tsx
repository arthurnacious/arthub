import { useQuery } from '@tanstack/react-query'
import { fetchProjects } from '../../lib/api/projects'
import { StatusBadge } from '../StatusBadge'
import { Skeleton } from '../Skeleton'

// Its own query, isolated from every other dashboard card - if this
// fails, the rest of the dashboard still renders fine. The error badge
// doubles as a retry button - there's no room for a full ErrorState in a
// one-line stat row.
export function ProjectsStatCard() {
  const { data, isPending, isError, refetch } = useQuery({
    queryKey: ['projects-count'],
    queryFn: () => fetchProjects({ page: 1, pageSize: 1, search: '' }),
  })

  return (
    <li className="flex items-center justify-between">
      <span className="text-[var(--text-dim)]">projects</span>
      {isPending ? (
        <Skeleton className="h-4 w-8" />
      ) : isError ? (
        <button type="button" onClick={() => refetch()} aria-label="Retry">
          <StatusBadge label="error" tone="danger" />
        </button>
      ) : (
        <StatusBadge label={String(data?.total ?? 0)} tone="info" />
      )}
    </li>
  )
}
