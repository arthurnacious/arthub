import { createFileRoute } from '@tanstack/react-router'
import { KernelLog } from '../components/KernelLog'
import { Panel } from '../components/Panel'
import { StatusBadge } from '../components/StatusBadge'

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

function Home() {
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
      </div>
    </div>
  )
}
