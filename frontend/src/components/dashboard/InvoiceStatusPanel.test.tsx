import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { InvoiceStatusPanel } from './InvoiceStatusPanel'
import * as invoicesApi from '../../lib/api/invoices'
import type { Invoice } from '../../lib/api/invoices'

function renderPanel() {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={queryClient}>
      <InvoiceStatusPanel />
    </QueryClientProvider>,
  )
}

const BASE_INVOICE: Invoice = {
  id: '1',
  invoiceNumber: 'INV-0001',
  projectId: 'p1',
  projectName: 'Site',
  clientId: 'c1',
  clientName: 'Acme',
  quoteId: null,
  status: 'draft',
  issueDate: null,
  dueDate: null,
  paidAt: null,
  total: 100,
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('InvoiceStatusPanel', () => {
  it('counts invoices per status', async () => {
    vi.spyOn(invoicesApi, 'fetchInvoices').mockResolvedValue({
      data: [
        { ...BASE_INVOICE, id: '1', status: 'paid' },
        { ...BASE_INVOICE, id: '2', status: 'paid' },
        { ...BASE_INVOICE, id: '3', status: 'draft' },
      ],
      page: 1,
      pageSize: 200,
      total: 3,
    })

    renderPanel()

    const chart = await screen.findByRole('img')
    const label = chart.getAttribute('aria-label') ?? ''
    expect(label).toContain('paid 2')
    expect(label).toContain('draft 1')
  })

  it('renders an empty ring when there are no invoices at all', async () => {
    vi.spyOn(invoicesApi, 'fetchInvoices').mockResolvedValue({
      data: [],
      page: 1,
      pageSize: 200,
      total: 0,
    })

    renderPanel()

    expect(await screen.findByRole('img')).toBeInTheDocument()
  })

  it('shows an error state with a working retry', async () => {
    const user = userEvent.setup()
    vi.spyOn(invoicesApi, 'fetchInvoices')
      .mockRejectedValueOnce(new Error('boom'))
      .mockResolvedValue({ data: [], page: 1, pageSize: 200, total: 0 })

    renderPanel()
    expect(await screen.findByText("COULDN'T LOAD INVOICE STATUS")).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'retry' }))

    expect(await screen.findByRole('img')).toBeInTheDocument()
  })
})
