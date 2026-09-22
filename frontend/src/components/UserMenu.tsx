import { Link } from '@tanstack/react-router'
import { useRef, useState } from 'react'
import { useDismissable } from '../hooks/use-dismissable'
import { StatusBadge } from './StatusBadge'
import { MOCK_CURRENT_USER } from '../lib/current-user'

const MENU_LINK_CLASSES =
  'rounded px-2 py-1 text-left text-[var(--text-dim)] transition-colors hover:bg-[var(--accent-dim)] hover:text-[var(--accent)]'

export function UserMenu() {
  const [open, setOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)
  useDismissable(containerRef, open, () => setOpen(false))

  function close() {
    setOpen(false)
  }

  return (
    <div ref={containerRef} className="relative">
      <button
        type="button"
        onClick={() => setOpen((value) => !value)}
        aria-expanded={open}
        className="rounded border border-[var(--border)] px-2 py-1 text-xs text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]"
      >
        {MOCK_CURRENT_USER.name}
      </button>
      {open && (
        <div className="glow-border absolute right-0 z-50 mt-2 w-64 rounded border border-[var(--border)] bg-[var(--panel)] p-3 text-sm">
          <div className="font-bold text-[var(--text)]">{MOCK_CURRENT_USER.name}</div>
          <div className="mt-0.5 text-[var(--text-dim)]">{MOCK_CURRENT_USER.email}</div>
          <div className="mt-2">
            <StatusBadge label={MOCK_CURRENT_USER.role} tone="info" />
          </div>
          <div className="my-3 border-t border-[var(--border)]" />
          <nav className="flex flex-col gap-1">
            <Link to="/profile" className={MENU_LINK_CLASSES} onClick={close}>
              &gt; profile
            </Link>
            <Link to="/invoices" className={MENU_LINK_CLASSES} onClick={close}>
              &gt; my invoices
            </Link>
            {/* Stub until a real session exists to log out of - see
                MOCK_CURRENT_USER. For now it just closes the menu. */}
            <button
              type="button"
              onClick={close}
              className="rounded px-2 py-1 text-left text-red-400 transition-colors hover:bg-red-500/10"
            >
              &gt; logout
            </button>
          </nav>
        </div>
      )}
    </div>
  )
}
