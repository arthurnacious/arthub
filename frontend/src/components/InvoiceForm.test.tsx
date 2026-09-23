import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { InvoiceForm } from './InvoiceForm'
import type { InvoiceInput } from '../lib/api/invoices'
import * as projectsApi from '../lib/api/projects'
import * as quotesApi from '../lib/api/quotes'

const EMPTY: InvoiceInput = {
  projectId: '',
  quoteId: null,
  status: 'draft',
  issueDate: null,
  dueDate: null,
  discountType: null,
  discountValue: 0,
  notes: '',
  lineItems: [],
}

function renderForm(props: Partial<Parameters<typeof InvoiceForm>[0]> = {}) {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const onChange = vi.fn()
  const onSubmit = vi.fn()
  const onCancel = vi.fn()

  const result = render(
    <QueryClientProvider client={queryClient}>
      <InvoiceForm
        values={EMPTY}
        onChange={onChange}
        onSubmit={onSubmit}
        onCancel={onCancel}
        submitLabel="create"
        submitting={false}
        {...props}
      />
    </QueryClientProvider>,
  )

  return { ...result, onChange, onSubmit, onCancel }
}

beforeEach(() => {
  vi.spyOn(projectsApi, 'fetchProjects').mockResolvedValue({
    data: [
      {
        id: 'p1',
        clientId: 'c1',
        clientName: 'Nimbus Retail',
        name: 'Marketing Site',
        description: '',
        category: 'web',
        url: null,
        status: 'active',
        coverImage: null,
      },
    ],
    page: 1,
    pageSize: 200,
    total: 1,
  })
  vi.spyOn(quotesApi, 'fetchQuotes').mockResolvedValue({
    data: [
      {
        id: 'q1',
        quoteNumber: 'Q-0001',
        projectId: 'p1',
        projectName: 'Marketing Site',
        clientId: 'c1',
        clientName: 'Nimbus Retail',
        status: 'accepted',
        issueDate: null,
        expiryDate: null,
        total: 200,
      },
    ],
    page: 1,
    pageSize: 200,
    total: 1,
  })
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('InvoiceForm', () => {
  it('lists fetched projects in the project select', async () => {
    renderForm()

    expect(await screen.findByRole('option', { name: 'Marketing Site (Nimbus Retail)' })).toBeInTheDocument()
  })

  it('lists fetched quotes in the quote select, defaulting to none', async () => {
    renderForm()

    expect(await screen.findByRole('option', { name: 'Q-0001 (Marketing Site)' })).toBeInTheDocument()
    expect(screen.getByRole('option', { name: 'none' })).toBeInTheDocument()
  })

  it('disables submit until a project is selected', () => {
    renderForm()

    expect(screen.getByRole('button', { name: 'create' })).toBeDisabled()
  })

  it('enables submit once a project is selected', () => {
    renderForm({ values: { ...EMPTY, projectId: 'p1' } })

    expect(screen.getByRole('button', { name: 'create' })).toBeEnabled()
  })

  it('shows a live total that updates as line items change', () => {
    renderForm({
      values: {
        ...EMPTY,
        projectId: 'p1',
        lineItems: [{ description: 'Design', quantity: 2, unitPrice: 100, discountType: null, discountValue: 0 }],
      },
    })

    expect(screen.getByText('total: 200.00')).toBeInTheDocument()
  })

  it('reflects the overall discount in the shown total', () => {
    renderForm({
      values: {
        ...EMPTY,
        projectId: 'p1',
        discountType: 'fixed',
        discountValue: 50,
        lineItems: [{ description: 'Design', quantity: 2, unitPrice: 100, discountType: null, discountValue: 0 }],
      },
    })

    expect(screen.getByText('total: 150.00')).toBeInTheDocument()
  })

  it('shows a hint about automatic paid_at only when status is paid', () => {
    const { rerender } = renderForm({ values: { ...EMPTY, projectId: 'p1' } })
    expect(screen.queryByText(/stamps today's date/i)).not.toBeInTheDocument()

    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    rerender(
      <QueryClientProvider client={queryClient}>
        <InvoiceForm
          values={{ ...EMPTY, projectId: 'p1', status: 'paid' }}
          onChange={() => {}}
          onSubmit={() => {}}
          onCancel={() => {}}
          submitLabel="create"
          submitting={false}
        />
      </QueryClientProvider>,
    )
    expect(screen.getByText(/stamps today's date/i)).toBeInTheDocument()
  })

  it('calls onSubmit when the form is submitted', async () => {
    const user = userEvent.setup()
    const { onSubmit } = renderForm({ values: { ...EMPTY, projectId: 'p1' } })

    await user.click(screen.getByRole('button', { name: 'create' }))

    expect(onSubmit).toHaveBeenCalledOnce()
  })

  it('calls onCancel when cancel is clicked', async () => {
    const user = userEvent.setup()
    const { onCancel } = renderForm()

    await user.click(screen.getByRole('button', { name: 'cancel' }))

    expect(onCancel).toHaveBeenCalledOnce()
  })
})
