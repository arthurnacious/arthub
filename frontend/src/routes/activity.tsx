import { useEffect, useState } from 'react'
import { createFileRoute, redirect } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { Panel } from '../components/Panel'
import { AuditLogTable } from '../components/AuditLogTable'
import { Pagination } from '../components/Pagination'
import { ErrorState } from '../components/ErrorState'
import { fetchActivityLog } from '../lib/api/activity'
import { useDebouncedValue } from '../hooks/use-debounced-value'
import { authStore } from '../stores/auth-store'

const PAGE_SIZE = 20

interface ActivitySearch {
  page: number
  q: string
}

// Same client-only auth guard as clients.tsx/projects.tsx/etc.
export const Route = createFileRoute('/activity')({
  validateSearch: (search: Record<string, unknown>): ActivitySearch => ({
    page: Number(search.page) > 0 ? Math.floor(Number(search.page)) : 1,
    q: typeof search.q === 'string' ? search.q : '',
  }),
  beforeLoad: () => {
    if (typeof window === 'undefined') return
    if (authStore.store.state.token === null) {
      throw redirect({ to: '/login' })
    }
  },
  component: ActivityPage,
})

// Full, searchable/paginated activity log - the dashboard's audit_log
// panel shows only the 5 most recent; this is the whole history.
function ActivityPage() {
  const { page, q } = Route.useSearch()
  const navigate = Route.useNavigate()

  const [searchInput, setSearchInput] = useState(q)
  const debouncedSearch = useDebouncedValue(searchInput, 300)

  useEffect(() => {
    if (debouncedSearch === q) return
    navigate({
      search: (prev) => ({ ...prev, q: debouncedSearch, page: 1 }),
      replace: true,
    })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debouncedSearch])

  const { data, isPending, isError, refetch } = useQuery({
    queryKey: ['activity-log', { page, pageSize: PAGE_SIZE, search: q }],
    queryFn: () => fetchActivityLog({ page, pageSize: PAGE_SIZE, search: q }),
    placeholderData: (previousData) => previousData,
  })

  return (
    <div>
      <h1 className="glow-text text-lg font-bold text-[var(--accent)]">ACTIVITY</h1>
      <p className="mt-2 text-sm text-[var(--text-dim)]">// everything that's happened, newest first</p>

      <div className="mt-6">
        <Panel title="activity_log">
          <input
            type="text"
            value={searchInput}
            onChange={(event) => setSearchInput(event.target.value)}
            placeholder="search action or description..."
            className="mb-4 w-full rounded border border-[var(--border)] bg-transparent px-3 py-1.5 text-sm text-[var(--text)] outline-none placeholder:text-[var(--text-dim)] focus:border-[var(--accent)]"
          />
          {isError ? (
            <ErrorState
              title="COULDN'T LOAD ACTIVITY"
              message="Something went wrong fetching the activity log."
              onRetry={() => refetch()}
            />
          ) : (
            <>
              <AuditLogTable entries={data?.data ?? []} isLoading={isPending} />
              <div className="mt-4">
                <Pagination
                  page={page}
                  pageSize={PAGE_SIZE}
                  total={data?.total ?? 0}
                  onPageChange={(nextPage) =>
                    navigate({ search: (prev) => ({ ...prev, page: nextPage }) })
                  }
                />
              </div>
            </>
          )}
        </Panel>
      </div>
    </div>
  )
}
