import { useQuery } from '@tanstack/react-query'
import { fetchClients } from '../../lib/api/clients'
import { StatusBadge } from '../StatusBadge'

// Its own query, isolated from every other dashboard card - if this
// fails, the rest of the dashboard still renders fine.
export function ClientsStatCard() {
  const { data, isLoading, isError } = useQuery({
    queryKey: ['clients-count'],
    queryFn: () => fetchClients({ page: 1, pageSize: 1, search: '' }),
  })

  return (
    <li className="flex items-center justify-between">
      <span className="text-[var(--text-dim)]">clients</span>
      {isLoading ? (
        <span className="text-xs text-[var(--text-dim)]">...</span>
      ) : isError ? (
        <StatusBadge label="error" tone="danger" />
      ) : (
        <StatusBadge label={String(data?.total ?? 0)} tone="info" />
      )}
    </li>
  )
}
