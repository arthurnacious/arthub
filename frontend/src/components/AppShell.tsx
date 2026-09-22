import { Link } from '@tanstack/react-router'
import type { ReactNode } from 'react'
import { ROUTES } from '../lib/routes'
import { useVimNavigation } from '../hooks/use-vim-navigation'
import { Mnemonic } from './Mnemonic'
import { Logo } from './Logo'
import { TopBar } from './TopBar'

// Terminal-style app shell: a bordered sidebar of route links plus the
// routed page content. Wraps every route via src/routes/__root.tsx.
//
// The sidebar doubles as a vim-style keyboard nav: j/k/g/G move a selection
// (the left accent bar) over these same routes, l/Enter follows it, h goes
// back. See src/hooks/use-vim-navigation.ts. The full shortcut list lives
// in the StatusBar footer.
export function AppShell({ children }: { children: ReactNode }) {
  const { selectedIndex } = useVimNavigation()

  return (
    <div className="flex flex-1 overflow-hidden">
      <aside className="flex w-56 shrink-0 flex-col overflow-y-auto border-r border-[var(--border)] bg-[var(--panel)]">
        <div className="flex h-14 shrink-0 items-center border-b border-[var(--border)] px-4">
          <Logo className="text-sm" />
        </div>
        <nav className="flex flex-col gap-0.5 p-2">
          {ROUTES.map((route, index) => (
            <Link
              key={route.path}
              to={route.path}
              activeOptions={{ exact: route.path === '/' }}
              className={`rounded-sm border-l-2 border-transparent px-2 py-1.5 text-sm transition-colors hover:bg-[var(--accent-dim)] ${
                index === selectedIndex ? 'border-[var(--accent)]' : ''
              }`}
              activeProps={{
                className: 'border-[var(--accent)] bg-[var(--accent-dim)]',
              }}
            >
              <Mnemonic text={route.name} />
            </Link>
          ))}
        </nav>
      </aside>
      <div className="flex flex-1 flex-col overflow-hidden">
        <TopBar />
        <main className="flex-1 overflow-y-auto p-8">{children}</main>
      </div>
    </div>
  )
}
