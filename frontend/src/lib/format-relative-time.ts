// Short "2m ago" style relative time. `now` is a parameter (not `new
// Date()` internally) so it's actually deterministic to test.
export function formatRelativeTime(iso: string, now: Date = new Date()): string {
  const diffSec = Math.round((now.getTime() - new Date(iso).getTime()) / 1000)

  if (diffSec < 60) return 'just now'

  const diffMin = Math.round(diffSec / 60)
  if (diffMin < 60) return `${diffMin}m ago`

  const diffHour = Math.round(diffMin / 60)
  if (diffHour < 24) return `${diffHour}h ago`

  const diffDay = Math.round(diffHour / 24)
  return `${diffDay}d ago`
}
