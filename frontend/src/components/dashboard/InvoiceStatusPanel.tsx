import { useQuery } from '@tanstack/react-query'
import { PieChart } from '../charts/PieChart'
import { ErrorState } from '../ErrorState'
import { Skeleton } from '../Skeleton'
import { INVOICE_STATUS_TONE, fetchInvoices } from '../../lib/api/invoices'
import type { InvoiceStatus } from '../../lib/api/invoices'
import type { PieDatum } from '../charts/PieChart'

// pageSize large enough to cover this app's scale in one request - same
// convention as RevenuePanel and the client/project/quote pickers.
const PAGE_SIZE = 200

// Its own query, isolated from every other dashboard panel - a count of
// invoices per status, across every invoice (not just paid ones).
export function InvoiceStatusPanel() {
  const { data, isPending, isError, refetch } = useQuery({
    queryKey: ['invoices', { page: 1, pageSize: PAGE_SIZE, search: '' }],
    queryFn: () => fetchInvoices({ page: 1, pageSize: PAGE_SIZE, search: '' }),
  })

  if (isPending) {
    return (
      <div className="flex items-center gap-4">
        <Skeleton className="h-[120px] w-[120px] rounded-full" />
        <div className="flex flex-col gap-2">
          <Skeleton className="h-3 w-20" />
          <Skeleton className="h-3 w-16" />
          <Skeleton className="h-3 w-20" />
        </div>
      </div>
    )
  }

  if (isError) {
    return (
      <ErrorState
        title="COULDN'T LOAD INVOICE STATUS"
        message="Something went wrong fetching invoices."
        onRetry={() => refetch()}
      />
    )
  }

  const counts = new Map<InvoiceStatus, number>()
  for (const invoice of data?.data ?? []) {
    counts.set(invoice.status, (counts.get(invoice.status) ?? 0) + 1)
  }

  const slices: PieDatum[] = Array.from(counts.entries()).map(([status, value]) => ({
    label: status,
    value,
    tone: INVOICE_STATUS_TONE[status],
  }))

  return <PieChart data={slices} />
}
