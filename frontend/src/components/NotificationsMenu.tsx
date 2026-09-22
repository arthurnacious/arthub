import { useRef, useState } from 'react'
import { useNavigate } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { useDismissable } from '../hooks/use-dismissable'
import { fetchActivity } from '../lib/api/activity'
import { activityLink } from '../lib/activity-link'
import { formatRelativeTime } from '../lib/format-relative-time'
import type { ActivityEntry } from '../lib/api/activity'

const ENTRY_BASE_CLASSES = 'flex items-start justify-between gap-2 rounded px-1 py-0.5 text-left'

export function NotificationsMenu() {
  const [open, setOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)
  const navigate = useNavigate()
  useDismissable(containerRef, open, () => setOpen(false))

  const { data } = useQuery({ queryKey: ['activity-log'], queryFn: fetchActivity })
  const entries = data ?? []

  function goToEntry(entry: ActivityEntry) {
    const link = activityLink(entry)
    if (!link) return
    navigate(link)
    setOpen(false)
  }

  return (
    <div ref={containerRef} className="relative">
      <button
        type="button"
        onClick={() => setOpen((value) => !value)}
        aria-label={`Notifications (${entries.length})`}
        aria-expanded={open}
        className="flex items-center gap-1 rounded border border-[var(--border)] px-2 py-1 text-xs uppercase tracking-wider text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]"
      >
        alerts
        {entries.length > 0 && (
          <span className="rounded-full bg-[var(--accent)] px-1.5 text-[10px] font-bold text-black">
            {entries.length}
          </span>
        )}
      </button>
      {open && (
        <div className="glow-border absolute right-0 z-50 mt-2 w-72 rounded border border-[var(--border)] bg-[var(--panel)] p-3 text-sm">
          <div className="mb-2 text-xs font-bold uppercase tracking-widest text-[var(--text-dim)]">
            notifications
          </div>
          {entries.length === 0 ? (
            <p className="text-[var(--text-dim)]">-- no notifications --</p>
          ) : (
            <ul className="flex flex-col gap-1">
              {entries.map((entry) => {
                const link = activityLink(entry)
                const timestamp = formatRelativeTime(entry.created_at)

                return (
                  <li key={entry.id}>
                    {link ? (
                      <button
                        type="button"
                        onClick={() => goToEntry(entry)}
                        className={`${ENTRY_BASE_CLASSES} w-full transition-colors hover:bg-[var(--accent-dim)]`}
                      >
                        <span className="text-[var(--text)]">{entry.description}</span>
                        <span className="shrink-0 text-[var(--text-dim)]">{timestamp}</span>
                      </button>
                    ) : (
                      <div className={ENTRY_BASE_CLASSES}>
                        <span className="text-[var(--text-dim)]">{entry.description}</span>
                        <span className="shrink-0 text-[var(--text-dim)]">{timestamp}</span>
                      </div>
                    )}
                  </li>
                )
              })}
            </ul>
          )}
        </div>
      )}
    </div>
  )
}
