import { useQuery } from '@tanstack/react-query'
import { fetchProjects } from '../../lib/api/projects'
import { StatusBadge } from '../StatusBadge'

// Its own query, isolated from every other dashboard card - if this
// fails, the rest of the dashboard still renders fine.
export function ProjectsStatCard() {
  const { data, isPending, isError } = useQuery({
    queryKey: ['projects-count'],
    queryFn: () => fetchProjects({ page: 1, pageSize: 1, search: '' }),
  })

  return (
    <li className="flex items-center justify-between">
      <span className="text-[var(--text-dim)]">projects</span>
      {isPending ? (
        <span className="text-xs text-[var(--text-dim)]">...</span>
      ) : isError ? (
        <StatusBadge label="error" tone="danger" />
      ) : (
        <StatusBadge label={String(data?.total ?? 0)} tone="info" />
      )}
    </li>
  )
}
