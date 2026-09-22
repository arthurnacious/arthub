import { createFileRoute } from '@tanstack/react-router'
import { KernelLog } from '../components/KernelLog'

export const Route = createFileRoute('/')({ component: Home })

// Placeholder entries until the dashboard is wired to GET /activity-log.
const MOCK_ENTRIES = [
  { id: '1', timestamp: '09:14:02', message: 'system boot: arthub dashboard' },
  { id: '2', timestamp: '09:14:03', message: 'waiting for backend connection', level: 'warn' as const },
]

function Home() {
  return (
    <div>
      <h1 className="text-lg font-bold text-[var(--accent)]">DASHBOARD</h1>
      <p className="mt-2 text-sm text-[var(--text-dim)]">
        // coming soon: an overview of clients, projects and recent activity
      </p>
      <div className="mt-6">
        <KernelLog entries={MOCK_ENTRIES} />
      </div>
    </div>
  )
}
