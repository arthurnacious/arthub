import { afterEach, describe, expect, it, vi } from 'vitest'
import {
  createInvoice,
  deleteInvoice,
  fetchInvoice,
  fetchInvoices,
  updateInvoice,
} from './invoices'
import type { Invoice, InvoiceDetail, InvoiceInput } from './invoices'
import { ApiError } from './http'
import * as http from './http'

const INVOICE: Invoice = {
  id: '1',
  invoiceNumber: 'INV-0001',
  projectId: 'p1',
  projectName: 'Marketing Site',
  clientId: 'c1',
  clientName: 'Nimbus Retail',
  quoteId: null,
  status: 'draft',
  issueDate: null,
  dueDate: null,
  paidAt: null,
  total: 100,
}

const INVOICE_DETAIL: InvoiceDetail = {
  id: '1',
  invoiceNumber: 'INV-0001',
  projectId: 'p1',
  projectName: 'Marketing Site',
  clientId: 'c1',
  clientName: 'Nimbus Retail',
  quoteId: null,
  quoteNumber: null,
  status: 'draft',
  issueDate: null,
  dueDate: null,
  paidAt: null,
  discountType: null,
  discountValue: 0,
  notes: '',
  subtotal: 100,
  discount: 0,
  total: 100,
  lines: [],
}

const INPUT: InvoiceInput = {
  projectId: 'p1',
  quoteId: null,
  status: 'draft',
  issueDate: null,
  dueDate: null,
  discountType: null,
  discountValue: 0,
  notes: '',
  lineItems: [{ description: 'Design', quantity: 1, unitPrice: 100, discountType: null, discountValue: 0 }],
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('fetchInvoices', () => {
  it('requests the right query string and returns the result as-is', async () => {
    const paginated = { data: [INVOICE], page: 2, pageSize: 10, total: 15 }
    const apiGetSpy = vi.spyOn(http, 'apiGet').mockResolvedValue(paginated)

    const result = await fetchInvoices({ page: 2, pageSize: 10, search: 'nimbus' })

    expect(apiGetSpy).toHaveBeenCalledWith('/invoices?page=2&pageSize=10&q=nimbus')
    expect(result).toEqual(paginated)
  })

  it('includes status in the query string when given', async () => {
    const apiGetSpy = vi
      .spyOn(http, 'apiGet')
      .mockResolvedValue({ data: [], page: 1, pageSize: 10, total: 0 })

    await fetchInvoices({ page: 1, pageSize: 10, search: '', status: 'overdue' })

    expect(apiGetSpy).toHaveBeenCalledWith('/invoices?page=1&pageSize=10&q=&status=overdue')
  })
})

describe('fetchInvoice', () => {
  it('returns the invoice detail on success', async () => {
    vi.spyOn(http, 'apiGet').mockResolvedValue(INVOICE_DETAIL)

    expect(await fetchInvoice('1')).toEqual(INVOICE_DETAIL)
  })

  it('returns null (not a throw) on a 404', async () => {
    vi.spyOn(http, 'apiGet').mockRejectedValue(new ApiError('Invoice not found', 404))

    expect(await fetchInvoice('nonexistent')).toBeNull()
  })

  it('rethrows non-404 errors', async () => {
    vi.spyOn(http, 'apiGet').mockRejectedValue(new ApiError('Server error', 500))

    await expect(fetchInvoice('1')).rejects.toBeInstanceOf(ApiError)
  })
})

describe('createInvoice', () => {
  it('posts the input (including line items) and returns the created invoice', async () => {
    const apiPostSpy = vi.spyOn(http, 'apiPost').mockResolvedValue(INVOICE_DETAIL)

    const result = await createInvoice(INPUT)

    expect(apiPostSpy).toHaveBeenCalledWith('/invoices', INPUT)
    expect(result).toEqual(INVOICE_DETAIL)
  })
})

describe('updateInvoice', () => {
  it('patches the given id with the given fields', async () => {
    const apiPatchSpy = vi.spyOn(http, 'apiPatch').mockResolvedValue(INVOICE_DETAIL)

    const result = await updateInvoice('1', INPUT)

    expect(apiPatchSpy).toHaveBeenCalledWith('/invoices/1', INPUT)
    expect(result).toEqual(INVOICE_DETAIL)
  })
})

describe('deleteInvoice', () => {
  it('sends a DELETE for the given id', async () => {
    const apiDeleteSpy = vi.spyOn(http, 'apiDelete').mockResolvedValue(undefined)

    await deleteInvoice('1')

    expect(apiDeleteSpy).toHaveBeenCalledWith('/invoices/1')
  })
})
