export type KernelLogLevel = 'info' | 'warn' | 'error'

export interface KernelLogEntry {
  id: string
  timestamp: string
  message: string
  level?: KernelLogLevel
}

const LEVEL_CLASSES: Record<KernelLogLevel, string> = {
  info: 'text-[var(--accent)]',
  warn: 'text-amber-400',
  error: 'text-red-400',
}

// A scrolling, terminal-style feed of recent activity. Presentational only
// (fed mock entries for now, real ones from GET /activity-log once the
// frontend has auth wired up), and chrome-free - meant to sit inside a
// <Panel title="kernel_log">.
export function KernelLog({ entries }: { entries: KernelLogEntry[] }) {
  return (
    <>
      {entries.length === 0 ? (
        <p className="text-sm text-[var(--text-dim)]">-- no entries --</p>
      ) : (
        <ul className="flex flex-col gap-1 text-sm">
          {entries.map((entry) => (
            <li key={entry.id} className="flex gap-2">
              <span className="shrink-0 text-[var(--text-dim)]">
                [{entry.timestamp}]
              </span>
              <span className={LEVEL_CLASSES[entry.level ?? 'info']}>
                {entry.message}
              </span>
            </li>
          ))}
        </ul>
      )}
      <div className="mt-2 flex gap-1 text-sm text-[var(--accent)]">
        <span>&gt;</span>
        <span className="cursor-blink">_</span>
      </div>
    </>
  )
}
