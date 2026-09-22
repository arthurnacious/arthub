import { useEffect, useState } from 'react'
import { useLocation } from '@tanstack/react-router'
import { useStore } from '@tanstack/react-store'
import { themeStore } from '../stores/theme-store'

function formatClock(date: Date): string {
  return date.toLocaleTimeString('en-GB', { hour12: false })
}

// Bottom status line, terminal-dashboard style: app name/version, a live
// clock, the current route, the active theme "scheme" and a keyboard hint.
// The clock starts null and only ticks after mount, so server and client
// render the same markup on hydration (a live Date can't be SSR'd safely).
export function StatusBar() {
  const [now, setNow] = useState<Date | null>(null)
  const location = useLocation()
  const color = useStore(themeStore.store, (state) => state)

  useEffect(() => {
    setNow(new Date())
    const id = setInterval(() => setNow(new Date()), 1000)
    return () => clearInterval(id)
  }, [])

  return (
    <footer className="flex flex-wrap items-center justify-between gap-x-4 gap-y-1 border-t border-[var(--border)] bg-[var(--panel)] px-4 py-1 text-xs text-[var(--text-dim)]">
      <span>
        arthub <span className="text-[var(--accent)]">v0.1.0</span>
      </span>
      <span>{now ? formatClock(now) : '--:--:--'}</span>
      <span>
        route: <span className="text-[var(--text)]">{location.pathname}</span>
      </span>
      <span>
        scheme: <span className="text-[var(--text)]">{color}</span>
      </span>
      <span>j/k nav &middot; enter go &middot; h back &middot; : cmd</span>
    </footer>
  )
}
