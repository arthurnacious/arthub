import type { BarDatum } from '../components/charts/BarChart'

const MONTH_LABELS = [
  'jan', 'feb', 'mar', 'apr', 'may', 'jun',
  'jul', 'aug', 'sep', 'oct', 'nov', 'dec',
]

const MOCK_MONTHLY_VALUES = [
  1200, 1800, 900, 2600, 3100, 2400, 2900, 2100, 1700, 2000, 2600, 3300,
]

// Placeholder monthly revenue until invoices are wired to the backend. A
// rolling 12-month window ending at the current month (not a fixed
// Jan-Dec) - grouped by year, so it's clear which September is which when
// the window spans two calendar years.
export function generateMockRevenue(): Required<BarDatum>[] {
  const now = new Date()

  return Array.from({ length: 12 }, (_, i) => {
    const monthsAgo = 11 - i
    const date = new Date(now.getFullYear(), now.getMonth() - monthsAgo, 1)

    return {
      label: MONTH_LABELS[date.getMonth()],
      value: MOCK_MONTHLY_VALUES[i],
      group: String(date.getFullYear()),
      muted: false,
    }
  })
}
