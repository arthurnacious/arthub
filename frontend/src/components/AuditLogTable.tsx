import { useNavigate } from '@tanstack/react-router'
import { activityLink } from '../lib/activity-link'
import { formatRelativeTime } from '../lib/format-relative-time'
import { TableRowsSkeleton } from './TableRowsSkeleton'
import type { ActivityEntry } from '../lib/api/activity'

const SKELETON_COLUMNS = [
  { className: 'py-2 pr-4', width: 'w-16' },
  { className: 'hidden py-2 pr-4 sm:table-cell', width: 'w-24' },
  { className: 'py-2', width: 'w-64' },
]

// Presentational only - renders whatever page of entries it's given (see
// src/routes/index.tsx for where they come from). Rows with a known
// subject (activityLink() resolves one) are clickable and navigate there;
// others (e.g. auth events, which have no subject) render as plain text.
// isLoading renders skeleton rows under the same real header.
export function AuditLogTable({
  entries,
  isLoading = false,
}: {
  entries: ActivityEntry[]
  isLoading?: boolean
}) {
  const navigate = useNavigate()

  if (!isLoading && entries.length === 0) {
    return <p className="text-sm text-[var(--text-dim)]">-- no activity yet --</p>
  }

  return (
    <table className="w-full text-left text-sm">
      <thead>
        <tr className="border-b border-[var(--border)] text-xs uppercase tracking-wider text-[var(--text-dim)]">
          <th className="py-2 pr-4 font-normal">When</th>
          <th className="hidden py-2 pr-4 font-normal sm:table-cell">Action</th>
          <th className="py-2 font-normal">Description</th>
        </tr>
      </thead>
      <tbody>
        {isLoading ? (
          <TableRowsSkeleton columns={SKELETON_COLUMNS} />
        ) : (
          entries.map((entry) => {
            const link = activityLink(entry)
            const clickable = link !== null

            return (
              <tr
                key={entry.id}
                onClick={clickable ? () => navigate(link) : undefined}
                className={`border-b border-[var(--border)] transition-colors ${
                  clickable ? 'cursor-pointer hover:bg-[var(--accent-dim)]' : ''
                }`}
              >
                <td className="py-2 pr-4 whitespace-nowrap text-[var(--text-dim)]">
                  {formatRelativeTime(entry.created_at)}
                </td>
                <td className="hidden py-2 pr-4 text-[var(--text-dim)] sm:table-cell">
                  {entry.action}
                </td>
                <td className="py-2 text-[var(--text)]">{entry.description}</td>
              </tr>
            )
          })
        )}
      </tbody>
    </table>
  )
}
