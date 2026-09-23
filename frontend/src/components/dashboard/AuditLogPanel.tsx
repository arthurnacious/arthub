import { useEffect, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { AuditLogTable } from '../AuditLogTable'
import { Pagination } from '../Pagination'
import { fetchActivityLog } from '../../lib/api/activity'
import { useDebouncedValue } from '../../hooks/use-debounced-value'

const PAGE_SIZE = 5

// Owns its own search/pagination state and its own query - the parent
// dashboard just hands it the URL-derived page/q and a couple of
// callbacks to push changes back into the URL. Loading and error states
// stay local to this panel.
export function AuditLogPanel({
  page,
  q,
  onPageChange,
  onSearchChange,
}: {
  page: number
  q: string
  onPageChange: (page: number) => void
  onSearchChange: (q: string) => void
}) {
  const [searchInput, setSearchInput] = useState(q)
  const debouncedSearch = useDebouncedValue(searchInput, 300)

  useEffect(() => {
    if (debouncedSearch === q) return
    onSearchChange(debouncedSearch)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debouncedSearch])

  const { data, isPending, isError } = useQuery({
    queryKey: ['activity-log', { page, pageSize: PAGE_SIZE, search: q }],
    queryFn: () => fetchActivityLog({ page, pageSize: PAGE_SIZE, search: q }),
    placeholderData: (previousData) => previousData,
  })

  return (
    <>
      <input
        type="text"
        value={searchInput}
        onChange={(event) => setSearchInput(event.target.value)}
        placeholder="search action or description..."
        className="mb-4 w-full rounded border border-[var(--border)] bg-transparent px-3 py-1.5 text-sm text-[var(--text)] outline-none placeholder:text-[var(--text-dim)] focus:border-[var(--accent)]"
      />
      {isPending ? (
        <p className="text-sm text-[var(--text-dim)]">loading...</p>
      ) : isError ? (
        <p className="text-sm text-red-400">-- couldn't load the audit log --</p>
      ) : (
        <>
          <AuditLogTable entries={data?.data ?? []} />
          <div className="mt-4">
            <Pagination
              page={page}
              pageSize={PAGE_SIZE}
              total={data?.total ?? 0}
              onPageChange={onPageChange}
            />
          </div>
        </>
      )}
    </>
  )
}
