import { Skeleton } from './Skeleton'

export interface SkeletonColumn {
  /** Full <td> className, matching the real column's cell exactly (including responsive hidden/table-cell classes). */
  className: string
  /** Tailwind width class for the placeholder bar. */
  width: string
  align?: 'right'
}

// Shared skeleton <tbody> content for any table whose columns are known
// up front (ClientsTable/QuotesTable/InvoicesTable) - keeps the real
// <thead> in place (so column headers don't flicker in/out) while rows
// are still loading.
export function TableRowsSkeleton({
  rows = 5,
  columns,
}: {
  rows?: number
  columns: SkeletonColumn[]
}) {
  return (
    <>
      {Array.from({ length: rows }).map((_, row) => (
        <tr key={row} className="border-b border-[var(--border)]">
          {columns.map((column, col) => (
            <td key={col} className={column.className}>
              <Skeleton className={`h-4 ${column.width} ${column.align === 'right' ? 'ml-auto' : ''}`} />
            </td>
          ))}
        </tr>
      ))}
    </>
  )
}
