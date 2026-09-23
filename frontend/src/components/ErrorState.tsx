// A failed-fetch screen with a retry button - the counterpart to
// Skeleton for the other side of a query's lifecycle. `onRetry` is
// typically a TanStack Query `refetch`.
export function ErrorState({
  title,
  message,
  onRetry,
}: {
  title: string
  message: string
  onRetry: () => void
}) {
  return (
    <div className="flex flex-col items-start gap-3 rounded border border-red-500/30 bg-red-500/5 p-4">
      <div>
        <h2 className="text-sm font-bold uppercase tracking-wider text-red-400">{title}</h2>
        <p className="mt-1 text-sm text-[var(--text-dim)]">{message}</p>
      </div>
      <button
        type="button"
        onClick={onRetry}
        className="rounded border border-red-400/50 px-3 py-1.5 text-xs text-red-400 transition-colors hover:bg-red-500/10"
      >
        retry
      </button>
    </div>
  )
}
