import { Link } from '@tanstack/react-router'
import type { ReactNode } from 'react'
import { ROUTES } from '../lib/routes'

// Terminal-style app shell: a fixed sidebar of route links plus the routed
// page content. Wraps every route via src/routes/__root.tsx.
export function AppShell({ children }: { children: ReactNode }) {
  return (
    <div className="flex min-h-screen">
      <aside className="glow-border flex w-56 shrink-0 flex-col border-r border-[var(--border)] bg-[var(--panel)] p-4">
        <div className="glow-text mb-6 text-sm font-bold tracking-widest text-[var(--accent)]">
          ARTHUB<span className="cursor-blink text-[var(--accent)]">_</span>
        </div>
        <nav className="flex flex-col gap-1">
          {ROUTES.map((route) => (
            <Link
              key={route.path}
              to={route.path}
              activeOptions={{ exact: route.path === '/' }}
              className="rounded px-2 py-1 text-sm text-[var(--text-dim)] transition-colors hover:bg-[var(--accent-dim)] hover:text-[var(--text)]"
              activeProps={{
                className: 'bg-[var(--accent-dim)] text-[var(--accent)]',
              }}
            >
              &gt; {route.name}
            </Link>
          ))}
        </nav>
      </aside>
      <main className="flex-1 p-8">{children}</main>
    </div>
  )
}
