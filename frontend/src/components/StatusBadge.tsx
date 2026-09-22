export type StatusTone = 'success' | 'warning' | 'danger' | 'info' | 'neutral'

const TONE_CLASSES: Record<StatusTone, string> = {
  success: 'border-emerald-500/40 bg-emerald-500/10 text-emerald-400',
  warning: 'border-amber-500/40 bg-amber-500/10 text-amber-400',
  danger: 'border-red-500/40 bg-red-500/10 text-red-400',
  info: 'border-sky-500/40 bg-sky-500/10 text-sky-400',
  neutral: 'border-zinc-500/40 bg-zinc-500/10 text-zinc-400',
}

// Small pill for statuses like "paid" / "overdue" / "draft" across the app.
export function StatusBadge({
  label,
  tone = 'neutral',
}: {
  label: string
  tone?: StatusTone
}) {
  return (
    <span
      className={`inline-block rounded border px-2 py-0.5 text-xs uppercase tracking-wider ${TONE_CLASSES[tone]}`}
    >
      {label}
    </span>
  )
}
