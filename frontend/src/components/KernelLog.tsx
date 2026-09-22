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

// A scrolling, terminal-style panel of recent activity. Presentational only
// so it can be fed either mock entries (for now) or real ones from the
// /activity endpoint once the frontend has auth wired up.
export function KernelLog({ entries }: { entries: KernelLogEntry[] }) {
  return (
    <div className="rounded border border-[var(--border)] bg-[var(--panel)] p-4">
      <div className="mb-3 text-xs font-bold uppercase tracking-widest text-[var(--text-dim)]">
        kernel_log
      </div>
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
    </div>
  )
}
