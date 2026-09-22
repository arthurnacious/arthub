import { ThemeToggle } from './ThemeToggle'
import { NotificationsMenu } from './NotificationsMenu'
import { UserMenu } from './UserMenu'

export function TopBar() {
  return (
    <header className="flex items-center justify-end gap-3 border-b border-[var(--border)] bg-[var(--panel)] px-6 py-3">
      <ThemeToggle />
      <NotificationsMenu />
      <UserMenu />
    </header>
  )
}
