import { useRef } from 'react'
import type { ReactNode } from 'react'
import { useDismissable } from '../hooks/use-dismissable'

// Generic titled modal: backdrop, bordered panel, closes on Escape, an
// outside click, or its own close button. Content is whatever's passed as
// children (e.g. a form).
export function Modal({
  title,
  onClose,
  children,
}: {
  title: string
  onClose: () => void
  children: ReactNode
}) {
  const panelRef = useRef<HTMLDivElement>(null)
  useDismissable(panelRef, true, onClose)

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 p-4">
      <div
        ref={panelRef}
        className="glow-border w-full max-w-md rounded border border-[var(--border)] bg-[var(--panel)] p-4"
      >
        <div className="mb-4 flex items-center justify-between">
          <div className="text-sm font-bold uppercase tracking-widest text-[var(--accent)]">
            {title}
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close"
            className="rounded px-1.5 py-0.5 text-[var(--text-dim)] transition-colors hover:bg-[var(--accent-dim)] hover:text-[var(--accent)]"
          >
            &#10005;
          </button>
        </div>
        {children}
      </div>
    </div>
  )
}
