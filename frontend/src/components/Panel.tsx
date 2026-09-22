import type { ReactNode } from 'react'
import { Mnemonic } from './Mnemonic'

// A bordered, titled box - the building block of the panel-grid dashboard
// look (bordered sections with a mnemonic-highlighted title bar, like the
// reference "Clock"/"Weather"/"Stocks" panels).
export function Panel({
  title,
  children,
  className = '',
}: {
  title: string
  children: ReactNode
  className?: string
}) {
  return (
    <section
      className={`glow-border rounded border border-[var(--border)] bg-[var(--panel)] ${className}`}
    >
      <header className="border-b border-[var(--border)] px-3 py-2 text-xs">
        <Mnemonic text={title} />
      </header>
      <div className="p-4">{children}</div>
    </section>
  )
}
