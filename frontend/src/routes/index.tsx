import { createFileRoute, redirect } from '@tanstack/react-router'
import { Panel } from '../components/Panel'
import { ClientsStatCard } from '../components/dashboard/ClientsStatCard'
import { ProjectsStatCard } from '../components/dashboard/ProjectsStatCard'
import { AuditLogPanel } from '../components/dashboard/AuditLogPanel'
import { ActivityGraphPanel } from '../components/dashboard/ActivityGraphPanel'
import { RevenuePanel } from '../components/dashboard/RevenuePanel'
import { InvoiceStatusPanel } from '../components/dashboard/InvoiceStatusPanel'
import { authStore } from '../stores/auth-store'

interface DashboardSearch {
  page: number
  q: string
}

// Same client-only auth guard as clients.tsx/profile.tsx/etc - see those
// for why it's client-only (the token lives in localStorage).
export const Route = createFileRoute('/')({
  beforeLoad: () => {
    if (typeof window === 'undefined') return
    if (authStore.store.state.token === null) {
      throw redirect({ to: '/login' })
    }
  },
  validateSearch: (search: Record<string, unknown>): DashboardSearch => ({
    page: Number(search.page) > 0 ? Math.floor(Number(search.page)) : 1,
    q: typeof search.q === 'string' ? search.q : '',
  }),
  component: Home,
})

// Each panel below fetches (or generates) its own data independently -
// one panel's loading state or error never blocks or hides another's.
function Home() {
  const { page, q } = Route.useSearch()
  const navigate = Route.useNavigate()

  return (
    <div>
      <h1 className="glow-text text-lg font-bold text-[var(--accent)]">DASHBOARD</h1>
      <p className="mt-2 text-sm text-[var(--text-dim)]">
        // activity_graph below is still a placeholder - no daily activity aggregation endpoint yet
      </p>
      <div className="mt-6 grid gap-4 md:grid-cols-2">
        <Panel title="status">
          <ul className="flex flex-col gap-2 text-sm">
            <ClientsStatCard />
            <ProjectsStatCard />
          </ul>
        </Panel>
        <Panel title="audit_log" className="md:col-span-2">
          <AuditLogPanel
            page={page}
            q={q}
            onPageChange={(nextPage) =>
              navigate({ search: (prev) => ({ ...prev, page: nextPage }) })
            }
            onSearchChange={(nextQ) =>
              navigate({
                search: (prev) => ({ ...prev, q: nextQ, page: 1 }),
                replace: true,
              })
            }
          />
        </Panel>
        <Panel title="activity_graph" className="md:col-span-2">
          <ActivityGraphPanel />
        </Panel>
        <Panel title="revenue">
          <RevenuePanel />
        </Panel>
        <Panel title="invoice_status">
          <InvoiceStatusPanel />
        </Panel>
      </div>
    </div>
  )
}
