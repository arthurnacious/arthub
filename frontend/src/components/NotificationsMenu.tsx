import { useRef, useState } from 'react'
import { useDismissable } from '../hooks/use-dismissable'

interface Notification {
  id: string
  message: string
  timestamp: string
}

// Placeholder feed until this is wired to a real backend notifications
// endpoint. Structured the same way KernelLog's mock entries are.
const MOCK_NOTIFICATIONS: Notification[] = [
  { id: '1', message: 'quote Q-1042 was viewed by the client', timestamp: '2m ago' },
  { id: '2', message: 'invoice INV-0087 is now overdue', timestamp: '1h ago' },
]

export function NotificationsMenu() {
  const [open, setOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)
  useDismissable(containerRef, open, () => setOpen(false))

  return (
    <div ref={containerRef} className="relative">
      <button
        type="button"
        onClick={() => setOpen((value) => !value)}
        aria-label={`Notifications (${MOCK_NOTIFICATIONS.length} unread)`}
        aria-expanded={open}
        className="flex items-center gap-1 rounded border border-[var(--border)] px-2 py-1 text-xs uppercase tracking-wider text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]"
      >
        alerts
        {MOCK_NOTIFICATIONS.length > 0 && (
          <span className="rounded-full bg-[var(--accent)] px-1.5 text-[10px] font-bold text-black">
            {MOCK_NOTIFICATIONS.length}
          </span>
        )}
      </button>
      {open && (
        <div className="glow-border absolute right-0 z-50 mt-2 w-72 rounded border border-[var(--border)] bg-[var(--panel)] p-3 text-sm">
          <div className="mb-2 text-xs font-bold uppercase tracking-widest text-[var(--text-dim)]">
            notifications
          </div>
          {MOCK_NOTIFICATIONS.length === 0 ? (
            <p className="text-[var(--text-dim)]">-- no notifications --</p>
          ) : (
            <ul className="flex flex-col gap-2">
              {MOCK_NOTIFICATIONS.map((notification) => (
                <li key={notification.id} className="flex items-start justify-between gap-2">
                  <span className="text-[var(--text)]">{notification.message}</span>
                  <span className="shrink-0 text-[var(--text-dim)]">
                    {notification.timestamp}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  )
}
