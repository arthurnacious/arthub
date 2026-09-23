import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { QuoteForm } from './QuoteForm'
import type { QuoteInput } from '../lib/api/quotes'
import * as projectsApi from '../lib/api/projects'

const EMPTY: QuoteInput = {
  projectId: '',
  status: 'draft',
  issueDate: null,
  expiryDate: null,
  discountType: null,
  discountValue: 0,
  notes: '',
  lineItems: [],
}

function renderForm(props: Partial<Parameters<typeof QuoteForm>[0]> = {}) {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const onChange = vi.fn()
  const onSubmit = vi.fn()
  const onCancel = vi.fn()

  const result = render(
    <QueryClientProvider client={queryClient}>
      <QuoteForm
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
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('QuoteForm', () => {
  it('lists fetched projects in the project select', async () => {
    renderForm()

    expect(await screen.findByRole('option', { name: 'Marketing Site (Nimbus Retail)' })).toBeInTheDocument()
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

  it('reflects the quote-level discount in the shown total', () => {
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
