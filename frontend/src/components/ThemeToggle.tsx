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
      className="flex items-center gap-2 rounded border border-[var(--border)] px-2 py-1 text-xs uppercase tracking-wider text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]"
      aria-label={`Theme color: ${color}. Click to cycle.`}
    >
      <span
        className="h-2.5 w-2.5 shrink-0 rounded-full bg-[var(--accent)]"
        aria-hidden="true"
      />
      {color}
    </button>
  )
}
