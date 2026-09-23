import { StatusBadge } from './StatusBadge'
import { RowActionsMenu } from './RowActionsMenu'
import { TableRowsSkeleton } from './TableRowsSkeleton'
import { QUOTE_STATUS_TONE } from '../lib/api/quotes'
import type { Quote } from '../lib/api/quotes'

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

// Presentational only - renders whatever page of quotes it's given, and
// leaves what view/update/delete actually do to the caller. Same pattern
// as ClientsTable, including the isLoading skeleton mode.
export function QuotesTable({
  quotes,
  isLoading = false,
  onView,
  onUpdate,
  onDelete,
}: {
  quotes: Quote[]
  isLoading?: boolean
  onView: (quote: Quote) => void
  onUpdate: (quote: Quote) => void
  onDelete: (quote: Quote) => void
}) {
  if (!isLoading && quotes.length === 0) {
    return <p className="text-sm text-[var(--text-dim)]">-- no quotes match --</p>
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
          quotes.map((quote) => (
            <tr
              key={quote.id}
              className="border-b border-[var(--border)] transition-colors hover:bg-[var(--accent-dim)]"
            >
              <td className="py-2 pr-4 text-[var(--text)]">{quote.quoteNumber}</td>
              <td className="hidden py-2 pr-4 text-[var(--text-dim)] md:table-cell">{quote.projectName}</td>
              <td className="hidden py-2 pr-4 text-[var(--text-dim)] sm:table-cell">{quote.clientName}</td>
              <td className="py-2 pr-4">
                <StatusBadge label={quote.status} tone={QUOTE_STATUS_TONE[quote.status]} />
              </td>
              <td className="py-2 pr-4 text-right text-[var(--text)]">{formatCurrency(quote.total)}</td>
              <td className="py-2 text-right">
                <RowActionsMenu
                  item={quote}
                  label={quote.quoteNumber}
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
