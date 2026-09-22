import { useEffect, useMemo, useState } from 'react'
import { createFileRoute } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { AuditLogTable } from '../components/AuditLogTable'
import { Panel } from '../components/Panel'
import { Pagination } from '../components/Pagination'
import { StatusBadge } from '../components/StatusBadge'
import { ContributionGraph } from '../components/charts/ContributionGraph'
import { BarChart } from '../components/charts/BarChart'
import { PieChart } from '../components/charts/PieChart'
import { fetchActivityLog } from '../lib/api/activity'
import { generateMockActivity } from '../lib/mock-activity'
import { generateMockRevenue } from '../lib/mock-revenue'
import { useDebouncedValue } from '../hooks/use-debounced-value'

const AUDIT_LOG_PAGE_SIZE = 5

interface DashboardSearch {
  page: number
  q: string
}

export const Route = createFileRoute('/')({
  validateSearch: (search: Record<string, unknown>): DashboardSearch => ({
    page: Number(search.page) > 0 ? Math.floor(Number(search.page)) : 1,
    q: typeof search.q === 'string' ? search.q : '',
  }),
  component: Home,
})

// Placeholder counts until the dashboard is wired to real client/quote/
// invoice totals from the backend.
const MOCK_STATS = [
  { label: 'clients', value: 0, tone: 'info' as const },
  { label: 'open quotes', value: 0, tone: 'neutral' as const },
  { label: 'overdue invoices', value: 0, tone: 'danger' as const },
]

// Placeholder invoice status breakdown until invoices are wired to the backend.
const MOCK_INVOICE_STATUS = [
  { label: 'paid', value: 14, tone: 'success' as const },
  { label: 'pending', value: 5, tone: 'warning' as const },
  { label: 'overdue', value: 2, tone: 'danger' as const },
  { label: 'draft', value: 3, tone: 'neutral' as const },
]

function Home() {
  const { page, q } = Route.useSearch()
  const navigate = Route.useNavigate()

  // 53 weeks (371 days) so a full 12 months always show, ending on the
  // current month. Generated once per mount, not on every render - see
  // generateMockActivity.
  const activity = useMemo(() => generateMockActivity(371), [])
  const revenue = useMemo(() => generateMockRevenue(), [])

  // Local, immediately-responsive input state; the URL (and therefore the
  // server query below) only updates once typing settles.
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

  const { data: auditLog, isLoading: auditLogLoading } = useQuery({
    queryKey: ['activity-log', { page, pageSize: AUDIT_LOG_PAGE_SIZE, search: q }],
    queryFn: () => fetchActivityLog({ page, pageSize: AUDIT_LOG_PAGE_SIZE, search: q }),
    placeholderData: (previousData) => previousData,
  })

  return (
    <div>
      <h1 className="glow-text text-lg font-bold text-[var(--accent)]">DASHBOARD</h1>
      <p className="mt-2 text-sm text-[var(--text-dim)]">
        // coming soon: an overview of clients, projects and recent activity
      </p>
      <div className="mt-6 grid gap-4 md:grid-cols-2">
        <Panel title="status">
          <ul className="flex flex-col gap-2 text-sm">
            {MOCK_STATS.map((stat) => (
              <li key={stat.label} className="flex items-center justify-between">
                <span className="text-[var(--text-dim)]">{stat.label}</span>
                <StatusBadge label={String(stat.value)} tone={stat.tone} />
              </li>
            ))}
          </ul>
        </Panel>
        <Panel title="audit_log" className="md:col-span-2">
          <input
            type="text"
            value={searchInput}
            onChange={(event) => setSearchInput(event.target.value)}
            placeholder="search action or description..."
            className="mb-4 w-full rounded border border-[var(--border)] bg-transparent px-3 py-1.5 text-sm text-[var(--text)] outline-none placeholder:text-[var(--text-dim)] focus:border-[var(--accent)]"
          />
          {auditLogLoading ? (
            <p className="text-sm text-[var(--text-dim)]">loading...</p>
          ) : (
            <>
              <AuditLogTable entries={auditLog?.data ?? []} />
              <div className="mt-4">
                <Pagination
                  page={page}
                  pageSize={AUDIT_LOG_PAGE_SIZE}
                  total={auditLog?.total ?? 0}
                  onPageChange={(nextPage) =>
                    navigate({ search: (prev) => ({ ...prev, page: nextPage }) })
                  }
                />
              </div>
            </>
          )}
        </Panel>
        <Panel title="activity_graph" className="md:col-span-2">
          <ContributionGraph data={activity} />
        </Panel>
        <Panel title="revenue">
          <BarChart data={revenue} />
        </Panel>
        <Panel title="invoice_status">
          <PieChart data={MOCK_INVOICE_STATUS} />
        </Panel>
      </div>
    </div>
  )
}
