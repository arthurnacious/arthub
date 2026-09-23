import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { RevenuePanel } from './RevenuePanel'
import * as invoicesApi from '../../lib/api/invoices'
import type { Invoice } from '../../lib/api/invoices'

function renderPanel() {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={queryClient}>
      <RevenuePanel />
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
  status: 'paid',
  issueDate: null,
  dueDate: null,
  paidAt: null,
  total: 0,
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('RevenuePanel', () => {
  it('sums paid invoice totals falling in the same month', async () => {
    const today = new Date().toISOString()
    vi.spyOn(invoicesApi, 'fetchInvoices').mockResolvedValue({
      data: [
        { ...BASE_INVOICE, total: 500, paidAt: today },
        { ...BASE_INVOICE, id: '2', total: 300, paidAt: today },
      ],
      page: 1,
      pageSize: 200,
      total: 2,
    })

    renderPanel()

    const chart = await screen.findByRole('img')
    expect(chart.getAttribute('aria-label')).toContain('800')
  })

  it('ignores invoices that have not been paid', async () => {
    vi.spyOn(invoicesApi, 'fetchInvoices').mockResolvedValue({
      data: [{ ...BASE_INVOICE, total: 999, status: 'sent', paidAt: null }],
      page: 1,
      pageSize: 200,
      total: 1,
    })

    renderPanel()

    const chart = await screen.findByRole('img')
    const label = chart.getAttribute('aria-label') ?? ''
    expect(label).not.toContain('999')
    // Every one of the 12 months in the window has no paid revenue.
    expect(label.match(/no data/g)).toHaveLength(12)
  })

  it('shows an error state with a working retry', async () => {
    const user = userEvent.setup()
    vi.spyOn(invoicesApi, 'fetchInvoices')
      .mockRejectedValueOnce(new Error('boom'))
      .mockResolvedValue({ data: [], page: 1, pageSize: 200, total: 0 })

    renderPanel()
    expect(await screen.findByText("COULDN'T LOAD REVENUE")).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'retry' }))

    expect(await screen.findByRole('img')).toBeInTheDocument()
  })
})
