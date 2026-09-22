import { useRef, useState } from 'react'
import { useDismissable } from '../hooks/use-dismissable'

const ITEM_CLASSES =
  'block w-full rounded-sm px-2 py-1 text-left transition-colors text-[var(--text-dim)] hover:bg-[var(--accent-dim)] hover:text-[var(--accent)]'

// Generic per-row "..." menu (view/update/delete) - reusable across any
// data table (clients, and later projects/quotes/invoices).
export function RowActionsMenu<T>({
  item,
  label,
  onView,
  onUpdate,
  onDelete,
}: {
  item: T
  label: string
  onView: (item: T) => void
  onUpdate: (item: T) => void
  onDelete: (item: T) => void
}) {
  const [open, setOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)
  useDismissable(containerRef, open, () => setOpen(false))

  function act(handler: (item: T) => void) {
    handler(item)
    setOpen(false)
  }

  return (
    <div ref={containerRef} className="relative inline-block">
      <button
        type="button"
        onClick={() => setOpen((value) => !value)}
        aria-label={`Actions for ${label}`}
        aria-expanded={open}
        className="rounded px-1.5 py-0.5 leading-none text-[var(--text-dim)] transition-colors hover:bg-[var(--accent-dim)] hover:text-[var(--accent)]"
      >
        &#8942;
      </button>
      {open && (
        <div className="glow-border absolute right-0 z-50 mt-1 w-28 rounded border border-[var(--border)] bg-[var(--panel)] p-1 text-xs">
          <button type="button" onClick={() => act(onView)} className={ITEM_CLASSES}>
            view
          </button>
          <button type="button" onClick={() => act(onUpdate)} className={ITEM_CLASSES}>
            update
          </button>
          <button
            type="button"
            onClick={() => act(onDelete)}
            className="block w-full rounded-sm px-2 py-1 text-left text-red-400 transition-colors hover:bg-red-500/10"
          >
            delete
          </button>
        </div>
      )}
    </div>
  )
}
