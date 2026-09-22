import { useEffect } from 'react'

export interface ConfirmDialogProps {
  title?: string
  message: string
  confirmLabel?: string
  cancelLabel?: string
  tone?: 'default' | 'danger'
  onConfirm: () => void
  onCancel: () => void
}

// Presentational modal - see useConfirm for the promise-based hook that
// drives it. Escape cancels, same convention as the app's dropdowns.
export function ConfirmDialog({
  title = 'confirm',
  message,
  confirmLabel = 'confirm',
  cancelLabel = 'cancel',
  tone = 'default',
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') onCancel()
    }
    document.addEventListener('keydown', onKeyDown)
    return () => document.removeEventListener('keydown', onKeyDown)
  }, [onCancel])

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 p-4">
      <div className="glow-border w-full max-w-sm rounded border border-[var(--border)] bg-[var(--panel)] p-4">
        <div className="text-sm font-bold uppercase tracking-widest text-[var(--accent)]">
          {title}
        </div>
        <p className="mt-3 text-sm text-[var(--text-dim)]">{message}</p>
        <div className="mt-5 flex justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            className="rounded border border-[var(--border)] px-3 py-1.5 text-xs text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]"
          >
            {cancelLabel}
          </button>
          <button
            type="button"
            onClick={onConfirm}
            className={`rounded border px-3 py-1.5 text-xs transition-colors ${
              tone === 'danger'
                ? 'border-red-500/40 bg-red-500/10 text-red-400 hover:bg-red-500/20'
                : 'border-[var(--accent)] bg-[var(--accent-dim)] text-[var(--accent)]'
            }`}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  )
}
