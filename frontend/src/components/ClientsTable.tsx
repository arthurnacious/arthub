import { StatusBadge } from './StatusBadge'
import { RowActionsMenu } from './RowActionsMenu'
import { CLIENT_STATUS_TONE } from '../lib/mock-clients'
import type { Client } from '../lib/mock-clients'

// Presentational only - renders whatever page of clients it's given, and
// leaves what view/update/delete actually do to the caller. Where the
// page comes from (mock or real, paginated server-side) is likewise the
// caller's concern - see src/routes/clients.tsx.
export function ClientsTable({
  clients,
  onView,
  onUpdate,
  onDelete,
}: {
  clients: Client[]
  onView: (client: Client) => void
  onUpdate: (client: Client) => void
  onDelete: (client: Client) => void
}) {
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
          <th className="py-2 pr-4 text-right font-normal">Projects</th>
          <th className="py-2 text-right font-normal" aria-label="Actions" />
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
              <StatusBadge label={client.status} tone={CLIENT_STATUS_TONE[client.status]} />
            </td>
            <td className="py-2 pr-4 text-right text-[var(--text-dim)]">{client.projects}</td>
            <td className="py-2 text-right">
              <RowActionsMenu
                item={client}
                label={client.company}
                onView={onView}
                onUpdate={onUpdate}
                onDelete={onDelete}
              />
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  )
}
