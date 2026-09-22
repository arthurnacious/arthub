import type { ContributionDay } from '../components/charts/ContributionGraph'

function formatLocalDate(date: Date): string {
  const y = date.getFullYear()
  const m = String(date.getMonth() + 1).padStart(2, '0')
  const d = String(date.getDate()).padStart(2, '0')
  return `${y}-${m}-${d}`
}

// Placeholder activity history for the dashboard's contribution graph,
// until it's wired to real client/quote/invoice/activity-log data from the
// backend. Mostly quiet days with occasional bursts, like real activity.
export function generateMockActivity(days: number): ContributionDay[] {
  const today = new Date()
  const result: ContributionDay[] = []

  for (let i = days - 1; i >= 0; i--) {
    const date = new Date(today)
    date.setDate(date.getDate() - i)

    const roll = Math.random()
    const count =
      roll < 0.4
        ? 0
        : roll < 0.75
          ? 1 + Math.floor(Math.random() * 3)
          : 4 + Math.floor(Math.random() * 6)

    result.push({ date: formatLocalDate(date), count })
  }

  return result
}
