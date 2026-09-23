import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { InvoicesTable } from './InvoicesTable'
import type { Invoice } from '../lib/api/invoices'

const INVOICES: Invoice[] = [
  {
    id: '1',
    invoiceNumber: 'INV-0001',
    projectId: 'p1',
    projectName: 'Marketing Site',
    clientId: 'c1',
    clientName: 'Nimbus Retail',
    quoteId: null,
    status: 'sent',
    issueDate: null,
    dueDate: null,
    paidAt: null,
    total: 1234.5,
  },
]

describe('InvoicesTable', () => {
  it('shows a placeholder when there are no invoices', () => {
    render(<InvoicesTable invoices={[]} onView={() => {}} onUpdate={() => {}} onDelete={() => {}} />)

    expect(screen.getByText('-- no invoices match --')).toBeInTheDocument()
  })

  it('renders invoice number, project, client, status and formatted total', () => {
    render(<InvoicesTable invoices={INVOICES} onView={() => {}} onUpdate={() => {}} onDelete={() => {}} />)

    expect(screen.getByText('INV-0001')).toBeInTheDocument()
    expect(screen.getByText('Marketing Site')).toBeInTheDocument()
    expect(screen.getByText('Nimbus Retail')).toBeInTheDocument()
    expect(screen.getByText('sent')).toBeInTheDocument()
    expect(screen.getByText('1,234.50')).toBeInTheDocument()
  })

  it('calls onView when a row action selects view', async () => {
    const user = userEvent.setup()
    const onView = vi.fn()
    render(<InvoicesTable invoices={INVOICES} onView={onView} onUpdate={() => {}} onDelete={() => {}} />)

    await user.click(screen.getByRole('button', { name: /actions for inv-0001/i }))
    await user.click(screen.getByRole('button', { name: 'view' }))

    expect(onView).toHaveBeenCalledWith(INVOICES[0])
  })
})
