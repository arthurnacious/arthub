import { StatusBadge } from './StatusBadge'
import { RowActionsMenu } from './RowActionsMenu'
import { TableRowsSkeleton } from './TableRowsSkeleton'
import { CLIENT_STATUS_TONE } from '../lib/mock-clients'
import type { Client } from '../lib/mock-clients'

const SKELETON_COLUMNS = [
  { className: 'py-2 pr-4', width: 'w-32' },
  { className: 'hidden py-2 pr-4 md:table-cell', width: 'w-24' },
  { className: 'hidden py-2 pr-4 sm:table-cell', width: 'w-40' },
  { className: 'py-2 pr-4', width: 'w-16' },
  { className: 'hidden py-2 pr-4 text-right md:table-cell', width: 'w-6', align: 'right' as const },
  { className: 'py-2 text-right', width: 'w-4', align: 'right' as const },
]

// Presentational only - renders whatever page of clients it's given, and
// leaves what view/update/delete actually do to the caller. Where the
// page comes from (mock or real, paginated server-side) is likewise the
// caller's concern - see src/routes/clients.tsx. isLoading renders
// skeleton rows under the same real header, instead of the table
// disappearing and reappearing between fetches.
export function ClientsTable({
  clients,
  isLoading = false,
  onView,
  onUpdate,
  onDelete,
}: {
  clients: Client[]
  isLoading?: boolean
  onView: (client: Client) => void
  onUpdate: (client: Client) => void
  onDelete: (client: Client) => void
}) {
  if (!isLoading && clients.length === 0) {
    return <p className="text-sm text-[var(--text-dim)]">-- no clients match --</p>
  }

  return (
    <table className="w-full text-left text-sm">
      <thead>
        <tr className="border-b border-[var(--border)] text-xs uppercase tracking-wider text-[var(--text-dim)]">
          <th className="py-2 pr-4 font-normal">Company</th>
          <th className="hidden py-2 pr-4 font-normal md:table-cell">Contact</th>
          <th className="hidden py-2 pr-4 font-normal sm:table-cell">Email</th>
          <th className="py-2 pr-4 font-normal">Status</th>
          <th className="hidden py-2 pr-4 text-right font-normal md:table-cell">Projects</th>
          <th className="py-2 text-right font-normal" aria-label="Actions" />
        </tr>
      </thead>
      <tbody>
        {isLoading ? (
          <TableRowsSkeleton columns={SKELETON_COLUMNS} />
        ) : (
          clients.map((client) => (
            <tr
              key={client.id}
              className="border-b border-[var(--border)] transition-colors hover:bg-[var(--accent-dim)]"
            >
              <td className="py-2 pr-4 text-[var(--text)]">{client.company}</td>
              <td className="hidden py-2 pr-4 text-[var(--text-dim)] md:table-cell">{client.contact}</td>
              <td className="hidden py-2 pr-4 text-[var(--text-dim)] sm:table-cell">{client.email}</td>
              <td className="py-2 pr-4">
                <StatusBadge label={client.status} tone={CLIENT_STATUS_TONE[client.status]} />
              </td>
              <td className="hidden py-2 pr-4 text-right text-[var(--text-dim)] md:table-cell">
                {client.projects}
              </td>
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
          ))
        )}
      </tbody>
    </table>
  )
}
