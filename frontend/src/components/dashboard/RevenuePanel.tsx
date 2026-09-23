import { useMemo } from 'react'
import { BarChart } from '../charts/BarChart'
import { generateMockRevenue } from '../../lib/mock-revenue'

// No invoices backend yet - mock until real invoice totals exist.
// Self-contained so it never depends on (or blocks) any other dashboard
// panel.
export function RevenuePanel() {
  const revenue = useMemo(() => generateMockRevenue(), [])

  return <BarChart data={revenue} />
}
