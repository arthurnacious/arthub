// Local-time date helpers (not UTC / toISOString round-tripping), so a
// date never shifts a day depending on the viewer's timezone offset.
// Shared by anything that deals in plain YYYY-MM-DD dates rather than
// full timestamps - see ContributionGraph.tsx (where these originated)
// and ActivityGraphPanel.tsx.
export function parseLocalDate(iso: string): Date {
  const [y, m, d] = iso.split('-').map(Number)
  return new Date(y, m - 1, d)
}

export function formatLocalDate(date: Date): string {
  const y = date.getFullYear()
  const m = String(date.getMonth() + 1).padStart(2, '0')
  const d = String(date.getDate()).padStart(2, '0')
  return `${y}-${m}-${d}`
}
