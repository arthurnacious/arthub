import { useStore } from '@tanstack/react-store'
import { themeStore } from '../stores/theme-store'

// Cycles the terminal accent color (green -> purple -> orange -> red -> ...).
// Reads the label from the store itself, so it always shows the active color.
export function ThemeToggle() {
  const color = useStore(themeStore.store, (state) => state)

  return (
    <button
      type="button"
      onClick={() => themeStore.cycleColor()}
      className="border border-[var(--accent)] px-3 py-1 font-mono text-xs uppercase tracking-wider text-[var(--accent)] transition-colors hover:bg-[var(--accent-dim)]"
      aria-label={`Theme color: ${color}. Click to cycle.`}
    >
      {color}
    </button>
  )
}
