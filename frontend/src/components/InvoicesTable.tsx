import { StatusBadge } from './StatusBadge'
import { RowActionsMenu } from './RowActionsMenu'
import { TableRowsSkeleton } from './TableRowsSkeleton'
import { INVOICE_STATUS_TONE } from '../lib/api/invoices'
import type { Invoice } from '../lib/api/invoices'

function formatCurrency(value: number): string {
  return value.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

const SKELETON_COLUMNS = [
  { className: 'py-2 pr-4', width: 'w-20' },
  { className: 'hidden py-2 pr-4 md:table-cell', width: 'w-32' },
  { className: 'hidden py-2 pr-4 sm:table-cell', width: 'w-28' },
  { className: 'py-2 pr-4', width: 'w-16' },
  { className: 'py-2 pr-4 text-right', width: 'w-16', align: 'right' as const },
  { className: 'py-2 text-right', width: 'w-4', align: 'right' as const },
]

// Presentational only - renders whatever page of invoices it's given, and
// leaves what view/update/delete actually do to the caller. Same pattern
// as ClientsTable/QuotesTable, including the isLoading skeleton mode.
export function InvoicesTable({
  invoices,
  isLoading = false,
  onView,
  onUpdate,
  onDelete,
}: {
  invoices: Invoice[]
  isLoading?: boolean
  onView: (invoice: Invoice) => void
  onUpdate: (invoice: Invoice) => void
  onDelete: (invoice: Invoice) => void
}) {
  if (!isLoading && invoices.length === 0) {
    return <p className="text-sm text-[var(--text-dim)]">-- no invoices match --</p>
  }

  return (
    <table className="w-full text-left text-sm">
      <thead>
        <tr className="border-b border-[var(--border)] text-xs uppercase tracking-wider text-[var(--text-dim)]">
          <th className="py-2 pr-4 font-normal">Number</th>
          <th className="hidden py-2 pr-4 font-normal md:table-cell">Project</th>
          <th className="hidden py-2 pr-4 font-normal sm:table-cell">Client</th>
          <th className="py-2 pr-4 font-normal">Status</th>
          <th className="py-2 pr-4 text-right font-normal">Total</th>
          <th className="py-2 text-right font-normal" aria-label="Actions" />
        </tr>
      </thead>
      <tbody>
        {isLoading ? (
          <TableRowsSkeleton columns={SKELETON_COLUMNS} />
        ) : (
          invoices.map((invoice) => (
            <tr
              key={invoice.id}
              className="border-b border-[var(--border)] transition-colors hover:bg-[var(--accent-dim)]"
            >
              <td className="py-2 pr-4 text-[var(--text)]">{invoice.invoiceNumber}</td>
              <td className="hidden py-2 pr-4 text-[var(--text-dim)] md:table-cell">{invoice.projectName}</td>
              <td className="hidden py-2 pr-4 text-[var(--text-dim)] sm:table-cell">{invoice.clientName}</td>
              <td className="py-2 pr-4">
                <StatusBadge label={invoice.status} tone={INVOICE_STATUS_TONE[invoice.status]} />
              </td>
              <td className="py-2 pr-4 text-right text-[var(--text)]">{formatCurrency(invoice.total)}</td>
              <td className="py-2 text-right">
                <RowActionsMenu
                  item={invoice}
                  label={invoice.invoiceNumber}
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
