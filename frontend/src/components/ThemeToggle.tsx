import { useRef, useState } from 'react'
import { useStore } from '@tanstack/react-store'
import { themeStore, THEME_COLORS } from '../stores/theme-store'
import type { ThemeColor } from '../stores/theme-store'
import { useDismissable } from '../hooks/use-dismissable'

const SWATCH_CLASSES: Record<ThemeColor, string> = {
  green: 'bg-green-400',
  purple: 'bg-purple-400',
  orange: 'bg-orange-400',
  red: 'bg-red-400',
}

// Select-style dropdown for the terminal accent color: shows every option
// with its real swatch and picks one directly, instead of cycling blind
// through them one click at a time.
export function ThemeToggle() {
  const color = useStore(themeStore.store, (state) => state)
  const [open, setOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)
  useDismissable(containerRef, open, () => setOpen(false))

  function choose(next: ThemeColor) {
    themeStore.setColor(next)
    setOpen(false)
  }

  return (
    <div ref={containerRef} className="relative">
      <button
        type="button"
        onClick={() => setOpen((value) => !value)}
        aria-expanded={open}
        aria-label={`Theme color: ${color}`}
        className="flex items-center gap-2 rounded border border-[var(--border)] px-2 py-1 text-xs uppercase tracking-wider text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]"
      >
        <span
          className={`h-2.5 w-2.5 shrink-0 rounded-full ${SWATCH_CLASSES[color]}`}
          aria-hidden="true"
        />
        {color}
      </button>
      {open && (
        <div className="glow-border absolute right-0 z-50 mt-2 w-36 rounded border border-[var(--border)] bg-[var(--panel)] p-1 text-sm">
          {THEME_COLORS.map((option) => (
            <button
              key={option}
              type="button"
              onClick={() => choose(option)}
              className={`flex w-full items-center gap-2 rounded-sm px-2 py-1 text-left text-xs uppercase tracking-wider transition-colors hover:bg-[var(--accent-dim)] ${
                option === color ? 'text-[var(--accent)]' : 'text-[var(--text-dim)]'
              }`}
            >
              <span
                className={`h-2.5 w-2.5 shrink-0 rounded-full ${SWATCH_CLASSES[option]}`}
                aria-hidden="true"
              />
              {option}
              {option === color && <span className="ml-auto">&#10003;</span>}
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
