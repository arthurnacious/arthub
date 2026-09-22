import { Link } from '@tanstack/react-router'
import type { ReactNode } from 'react'
import { ROUTES } from '../lib/routes'
import { useVimNavigation } from '../hooks/use-vim-navigation'
import { TopBar } from './TopBar'

// Terminal-style app shell: a fixed sidebar of route links plus the routed
// page content. Wraps every route via src/routes/__root.tsx.
//
// The sidebar doubles as a vim-style keyboard nav: j/k/g/G move a selection
// ring over these same routes, l/Enter follows it, h goes back. See
// src/hooks/use-vim-navigation.ts.
export function AppShell({ children }: { children: ReactNode }) {
  const { selectedIndex } = useVimNavigation()

  return (
    <div className="flex min-h-screen">
      <aside className="glow-border flex w-56 shrink-0 flex-col border-r border-[var(--border)] bg-[var(--panel)] p-4">
        <div className="glow-text mb-6 text-sm font-bold tracking-widest text-[var(--accent)]">
          ARTHUB<span className="cursor-blink text-[var(--accent)]">_</span>
        </div>
        <nav className="flex flex-col gap-1">
          {ROUTES.map((route, index) => (
            <Link
              key={route.path}
              to={route.path}
              activeOptions={{ exact: route.path === '/' }}
              className={`rounded px-2 py-1 text-sm text-[var(--text-dim)] transition-colors hover:bg-[var(--accent-dim)] hover:text-[var(--text)] ${
                index === selectedIndex ? 'ring-1 ring-[var(--accent)]' : ''
              }`}
              activeProps={{
                className: 'bg-[var(--accent-dim)] text-[var(--accent)]',
              }}
            >
              &gt; {route.name}
            </Link>
          ))}
        </nav>
        <p className="mt-auto pt-4 text-xs text-[var(--text-dim)]">
          j/k select &middot; enter go &middot; h back &middot; : command
        </p>
      </aside>
      <div className="flex flex-1 flex-col">
        <TopBar />
        <main className="flex-1 p-8">{children}</main>
      </div>
    </div>
  )
}
