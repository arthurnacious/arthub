import { ThemeToggle } from './ThemeToggle'
import { NotificationsMenu } from './NotificationsMenu'
import { UserMenu } from './UserMenu'

export function TopBar() {
  return (
    <header className="flex h-14 shrink-0 items-center justify-end gap-3 border-b border-[var(--border)] bg-[var(--panel)] px-6">
      <ThemeToggle />
      <NotificationsMenu />
      <UserMenu />
    </header>
  )
}
