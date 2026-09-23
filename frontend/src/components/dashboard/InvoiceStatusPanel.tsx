import { PieChart } from '../charts/PieChart'

// No invoices backend yet - mock until real invoice statuses exist.
// Self-contained so it never depends on (or blocks) any other dashboard
// panel.
const MOCK_INVOICE_STATUS = [
  { label: 'paid', value: 14, tone: 'success' as const },
  { label: 'pending', value: 5, tone: 'warning' as const },
  { label: 'overdue', value: 2, tone: 'danger' as const },
  { label: 'draft', value: 3, tone: 'neutral' as const },
]

export function InvoiceStatusPanel() {
  return <PieChart data={MOCK_INVOICE_STATUS} />
}
