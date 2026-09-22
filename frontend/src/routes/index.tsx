import { useMemo } from 'react'
import { createFileRoute } from '@tanstack/react-router'
import { KernelLog } from '../components/KernelLog'
import { Panel } from '../components/Panel'
import { StatusBadge } from '../components/StatusBadge'
import { ContributionGraph } from '../components/charts/ContributionGraph'
import { BarChart } from '../components/charts/BarChart'
import { PieChart } from '../components/charts/PieChart'
import { generateMockActivity } from '../lib/mock-activity'
import { generateMockRevenue } from '../lib/mock-revenue'

export const Route = createFileRoute('/')({ component: Home })

// Placeholder entries until the dashboard is wired to GET /activity-log.
const MOCK_ENTRIES = [
  { id: '1', timestamp: '09:14:02', message: 'system boot: arthub dashboard' },
  { id: '2', timestamp: '09:14:03', message: 'waiting for backend connection', level: 'warn' as const },
]

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
  // 53 weeks (371 days) so a full 12 months always show, ending on the
  // current month. Generated once per mount, not on every render - see
  // generateMockActivity.
  const activity = useMemo(() => generateMockActivity(371), [])
  const revenue = useMemo(() => generateMockRevenue(), [])

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
        <Panel title="kernel_log">
          <KernelLog entries={MOCK_ENTRIES} />
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
