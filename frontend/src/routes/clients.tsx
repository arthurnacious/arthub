import { useEffect, useState } from 'react'
import { createFileRoute } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { Panel } from '../components/Panel'
import { ClientsTable } from '../components/ClientsTable'
import { Pagination } from '../components/Pagination'
import { fetchClients } from '../lib/api/clients'
import { useDebouncedValue } from '../hooks/use-debounced-value'

const PAGE_SIZE = 5

interface ClientsSearch {
  page: number
  q: string
}

export const Route = createFileRoute('/clients')({
  validateSearch: (search: Record<string, unknown>): ClientsSearch => ({
    page: Number(search.page) > 0 ? Math.floor(Number(search.page)) : 1,
    q: typeof search.q === 'string' ? search.q : '',
  }),
  component: ClientsPage,
})

function ClientsPage() {
  const { page, q } = Route.useSearch()
  const navigate = Route.useNavigate()

  // Local, immediately-responsive input state; the URL (and therefore the
  // server query below) only updates once typing settles - see
  // useDebouncedValue.
  const [searchInput, setSearchInput] = useState(q)
  const debouncedSearch = useDebouncedValue(searchInput, 300)

  useEffect(() => {
    if (debouncedSearch === q) return
    navigate({
      search: (prev) => ({ ...prev, q: debouncedSearch, page: 1 }),
      replace: true,
    })
    // Only the debounced value should drive this - navigating shouldn't
    // re-run it, and `q`/`navigate` are stable enough here that including
    // them would just cause redundant re-runs.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debouncedSearch])

  // page/q live in the URL and drive the query key, so each page or search
  // change is its own server request - not an in-memory filter over
  // everything already fetched.
  const { data, isLoading } = useQuery({
    queryKey: ['clients', { page, pageSize: PAGE_SIZE, search: q }],
    queryFn: () => fetchClients({ page, pageSize: PAGE_SIZE, search: q }),
    placeholderData: (previousData) => previousData,
  })

  return (
    <div>
      <h1 className="glow-text text-lg font-bold text-[var(--accent)]">CLIENTS</h1>
      <p className="mt-2 text-sm text-[var(--text-dim)]">
        // server-paginated - showing mock data until a real /clients endpoint exists
      </p>
      <div className="mt-6">
        <Panel title="clients">
          <input
            type="text"
            value={searchInput}
            onChange={(event) => setSearchInput(event.target.value)}
            placeholder="search company, contact or email..."
            className="mb-4 w-full rounded border border-[var(--border)] bg-transparent px-3 py-1.5 text-sm text-[var(--text)] outline-none placeholder:text-[var(--text-dim)] focus:border-[var(--accent)]"
          />
          {isLoading ? (
            <p className="text-sm text-[var(--text-dim)]">loading...</p>
          ) : (
            <>
              <ClientsTable clients={data?.data ?? []} />
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
