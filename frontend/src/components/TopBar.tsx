import { ThemeToggle } from './ThemeToggle'
import { NotificationsMenu } from './NotificationsMenu'
import { UserMenu } from './UserMenu'

export function TopBar({
  onMenuClick,
  mobileNavOpen,
}: {
  onMenuClick: () => void
  mobileNavOpen: boolean
}) {
  return (
    <header className="flex h-14 shrink-0 items-center justify-between gap-3 border-b border-[var(--border)] bg-[var(--panel)] px-4 md:justify-end md:px-6">
      <button
        type="button"
        onClick={onMenuClick}
        aria-label="Toggle navigation menu"
        aria-expanded={mobileNavOpen}
        className="rounded border border-[var(--border)] p-1.5 text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)] md:hidden"
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
          aria-hidden="true"
        >
          <line x1="2" y1="4" x2="14" y2="4" />
          <line x1="2" y1="8" x2="14" y2="8" />
          <line x1="2" y1="12" x2="14" y2="12" />
        </svg>
      </button>
      <div className="flex items-center gap-3">
        <ThemeToggle />
        <NotificationsMenu />
        <UserMenu />
      </div>
    </header>
  )
}
