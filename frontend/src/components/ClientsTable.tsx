import { StatusBadge } from './StatusBadge'
import type { StatusTone } from './StatusBadge'
import type { Client, ClientStatus } from '../lib/mock-clients'

const STATUS_TONE: Record<ClientStatus, StatusTone> = {
  active: 'success',
  prospect: 'info',
  inactive: 'neutral',
}

// Presentational only - renders whatever page of clients it's given.
// Where that page comes from (mock or real, paginated server-side) is the
// caller's concern - see src/routes/clients.tsx.
export function ClientsTable({ clients }: { clients: Client[] }) {
  if (clients.length === 0) {
    return <p className="text-sm text-[var(--text-dim)]">-- no clients match --</p>
  }

  return (
    <table className="w-full text-left text-sm">
      <thead>
        <tr className="border-b border-[var(--border)] text-xs uppercase tracking-wider text-[var(--text-dim)]">
          <th className="py-2 pr-4 font-normal">Company</th>
          <th className="py-2 pr-4 font-normal">Contact</th>
          <th className="py-2 pr-4 font-normal">Email</th>
          <th className="py-2 pr-4 font-normal">Status</th>
          <th className="py-2 text-right font-normal">Projects</th>
        </tr>
      </thead>
      <tbody>
        {clients.map((client) => (
          <tr
            key={client.id}
            className="border-b border-[var(--border)] transition-colors hover:bg-[var(--accent-dim)]"
          >
            <td className="py-2 pr-4 text-[var(--text)]">{client.company}</td>
            <td className="py-2 pr-4 text-[var(--text-dim)]">{client.contact}</td>
            <td className="py-2 pr-4 text-[var(--text-dim)]">{client.email}</td>
            <td className="py-2 pr-4">
              <StatusBadge label={client.status} tone={STATUS_TONE[client.status]} />
            </td>
            <td className="py-2 text-right text-[var(--text-dim)]">{client.projects}</td>
          </tr>
        ))}
      </tbody>
    </table>
  )
}
