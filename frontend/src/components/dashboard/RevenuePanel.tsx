import { useQuery } from '@tanstack/react-query'
import { BarChart } from '../charts/BarChart'
import { ErrorState } from '../ErrorState'
import { Skeleton } from '../Skeleton'
import { fetchInvoices } from '../../lib/api/invoices'
import type { BarDatum } from '../charts/BarChart'

const MONTH_LABELS = [
  'jan', 'feb', 'mar', 'apr', 'may', 'jun',
  'jul', 'aug', 'sep', 'oct', 'nov', 'dec',
]
const WINDOW_MONTHS = 12
// pageSize large enough to cover this app's scale in one request - same
// "just fetch everything" convention as the client/project/quote pickers
// elsewhere, not a paginated view.
const PAGE_SIZE = 200

function monthKey(date: Date): string {
  return `${date.getFullYear()}-${date.getMonth()}`
}

// Its own query, isolated from every other dashboard panel. Revenue is
// recognized when an invoice is actually marked paid (paidAt), not when
// it's issued - a rolling 12-month window ending at the current month,
// grouped by year like the old mock data was, so a paid-off month reads
// the same as before, just with real numbers.
export function RevenuePanel() {
  const { data, isPending, isError, refetch } = useQuery({
    queryKey: ['invoices', { page: 1, pageSize: PAGE_SIZE, search: '' }],
    queryFn: () => fetchInvoices({ page: 1, pageSize: PAGE_SIZE, search: '' }),
  })

  if (isPending) {
    return (
      <div className="flex h-24 items-end gap-2">
        {Array.from({ length: WINDOW_MONTHS }).map((_, i) => (
          <Skeleton key={i} className={`w-4 ${i % 3 === 0 ? 'h-1/3' : i % 3 === 1 ? 'h-2/3' : 'h-full'}`} />
        ))}
      </div>
    )
  }

  if (isError) {
    return (
      <ErrorState
        title="COULDN'T LOAD REVENUE"
        message="Something went wrong fetching paid invoices."
        onRetry={() => refetch()}
      />
    )
  }

  const now = new Date()
  const months = Array.from({ length: WINDOW_MONTHS }, (_, i) => {
    const monthsAgo = WINDOW_MONTHS - 1 - i
    return new Date(now.getFullYear(), now.getMonth() - monthsAgo, 1)
  })

  const totalsByMonth = new Map<string, number>()
  for (const invoice of data?.data ?? []) {
    if (!invoice.paidAt) continue
    const key = monthKey(new Date(invoice.paidAt))
    totalsByMonth.set(key, (totalsByMonth.get(key) ?? 0) + invoice.total)
  }

  const bars: BarDatum[] = months.map((date) => {
    const value = totalsByMonth.get(monthKey(date)) ?? 0
    return {
      label: MONTH_LABELS[date.getMonth()],
      value,
      group: String(date.getFullYear()),
      muted: value === 0,
    }
  })

  return <BarChart data={bars} />
}
