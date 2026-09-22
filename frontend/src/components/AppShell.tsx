import { Link } from '@tanstack/react-router'
import type { ReactNode } from 'react'
import { useRef, useState } from 'react'
import { ROUTES } from '../lib/routes'
import { useVimNavigation } from '../hooks/use-vim-navigation'
import { useDismissable } from '../hooks/use-dismissable'
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
//
// Below md, the sidebar is an off-canvas drawer (toggled from TopBar's
// menu button) rather than always-visible - see the responsive classes on
// <aside> below.
export function AppShell({ children }: { children: ReactNode }) {
  const { selectedIndex } = useVimNavigation()
  const [mobileNavOpen, setMobileNavOpen] = useState(false)
  const asideRef = useRef<HTMLElement>(null)
  useDismissable(asideRef, mobileNavOpen, () => setMobileNavOpen(false))

  return (
    <div className="flex flex-1 overflow-hidden">
      {mobileNavOpen && (
        <div className="fixed inset-0 z-40 bg-black/60 md:hidden" aria-hidden="true" />
      )}
      <aside
        ref={asideRef}
        className={`fixed inset-y-0 left-0 z-50 flex w-56 shrink-0 flex-col overflow-y-auto border-r border-[var(--border)] bg-[var(--panel)] transition-transform duration-200 md:static md:z-auto md:translate-x-0 ${
          mobileNavOpen ? 'translate-x-0' : '-translate-x-full'
        }`}
      >
        <div className="flex h-14 shrink-0 items-center border-b border-[var(--border)] px-4">
          <Logo className="text-sm" />
        </div>
        <nav className="flex flex-col gap-0.5 p-2">
          {ROUTES.map((route, index) => (
            <Link
              key={route.path}
              to={route.path}
              activeOptions={{ exact: route.path === '/' }}
              onClick={() => setMobileNavOpen(false)}
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
        <TopBar
          onMenuClick={() => setMobileNavOpen((value) => !value)}
          mobileNavOpen={mobileNavOpen}
        />
        <main className="flex-1 overflow-y-auto p-8">{children}</main>
      </div>
    </div>
  )
}
