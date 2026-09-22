import { useRef, useState } from 'react'
import { useDismissable } from '../hooks/use-dismissable'

export interface Notification {
  id: string
  message: string
  timestamp: string
}

// Nothing generates a real notification yet - that's a future feature
// (e.g. an invoice being marked paid via online payment). This starts
// empty; `initialNotifications` exists so a real source can seed it later
// without changing this component, and so the clear/dismiss mechanics
// below are actually testable today.
export function NotificationsMenu({
  initialNotifications = [],
}: {
  initialNotifications?: Notification[]
} = {}) {
  const [open, setOpen] = useState(false)
  const [notifications, setNotifications] = useState(initialNotifications)
  const containerRef = useRef<HTMLDivElement>(null)
  useDismissable(containerRef, open, () => setOpen(false))

  function dismiss(id: string) {
    setNotifications((prev) => prev.filter((notification) => notification.id !== id))
  }

  function clearAll() {
    setNotifications([])
  }

  return (
    <div ref={containerRef} className="relative">
      <button
        type="button"
        onClick={() => setOpen((value) => !value)}
        aria-label={`Notifications${notifications.length > 0 ? ` (${notifications.length})` : ''}`}
        aria-expanded={open}
        className="flex items-center gap-1 rounded border border-[var(--border)] px-2 py-1 text-xs uppercase tracking-wider text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]"
      >
        alerts
        {notifications.length > 0 && (
          <span className="rounded-full bg-[var(--accent)] px-1.5 text-[10px] font-bold text-black">
            {notifications.length}
          </span>
        )}
      </button>
      {open && (
        <div className="glow-border absolute right-0 z-50 mt-2 w-72 rounded border border-[var(--border)] bg-[var(--panel)] p-3 text-sm">
          <div className="mb-2 flex items-center justify-between">
            <span className="text-xs font-bold uppercase tracking-widest text-[var(--text-dim)]">
              notifications
            </span>
            {notifications.length > 0 && (
              <button
                type="button"
                onClick={clearAll}
                className="text-xs text-[var(--text-dim)] transition-colors hover:text-[var(--accent)]"
              >
                clear all
              </button>
            )}
          </div>
          {notifications.length === 0 ? (
            <p className="text-[var(--text-dim)]">-- all caught up --</p>
          ) : (
            <ul className="flex flex-col gap-1">
              {notifications.map((notification) => (
                <li
                  key={notification.id}
                  className="flex items-start justify-between gap-2 rounded px-1 py-0.5"
                >
                  <span className="text-[var(--text)]">{notification.message}</span>
                  <span className="flex shrink-0 items-center gap-2">
                    <span className="text-[var(--text-dim)]">{notification.timestamp}</span>
                    <button
                      type="button"
                      onClick={() => dismiss(notification.id)}
                      aria-label={`Dismiss ${notification.message}`}
                      className="text-[var(--text-dim)] transition-colors hover:text-red-400"
                    >
                      &#10005;
                    </button>
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
