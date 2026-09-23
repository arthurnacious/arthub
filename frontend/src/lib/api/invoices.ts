import { ApiError, apiDelete, apiGet, apiPatch, apiPost } from './http'
import type { PaginatedResult, PaginationParams } from './pagination'
import type { StatusTone } from '../../components/StatusBadge'
import type { DiscountType, LineItem } from '../line-items'

export type { DiscountType } from '../line-items'
export type InvoiceStatus = 'draft' | 'sent' | 'paid' | 'overdue' | 'cancelled'

export const INVOICE_STATUS_TONE: Record<InvoiceStatus, StatusTone> = {
  draft: 'neutral',
  sent: 'info',
  paid: 'success',
  overdue: 'danger',
  cancelled: 'neutral',
}

// Matches InvoicesController::present() - see
// backend/src/Modules/Invoices/InvoicesController.php.
export interface Invoice {
  id: string
  invoiceNumber: string
  projectId: string
  projectName: string
  clientId: string
  clientName: string
  quoteId: string | null
  status: InvoiceStatus
  issueDate: string | null
  dueDate: string | null
  paidAt: string | null
  total: number
}

export interface InvoiceLine {
  id: string
  description: string
  quantity: number
  unitPrice: number
  discountType: DiscountType
  discountValue: number
  lineTotal: number
  position: number
}

// Matches InvoicesController::presentDetail().
export interface InvoiceDetail {
  id: string
  invoiceNumber: string
  projectId: string
  projectName: string
  clientId: string
  clientName: string
  quoteId: string | null
  quoteNumber: string | null
  status: InvoiceStatus
  issueDate: string | null
  dueDate: string | null
  paidAt: string | null
  discountType: DiscountType
  discountValue: number
  notes: string
  subtotal: number
  discount: number
  total: number
  lines: InvoiceLine[]
}

// A line item as sent to the backend - same shape as the shared LineItem
// type used by the editor UI.
export type InvoiceLineInput = LineItem

export interface InvoiceInput {
  projectId: string
  quoteId: string | null
  status: InvoiceStatus
  issueDate: string | null
  dueDate: string | null
  discountType: DiscountType
  discountValue: number
  notes: string
  lineItems: InvoiceLineInput[]
}

// GET /invoices?page=&pageSize=&q=&status= - server-side pagination,
// search (invoice number, project name, or client name) and status
// filter.
export function fetchInvoices({
  page,
  pageSize,
  search,
  status = '',
}: PaginationParams & { status?: string }): Promise<PaginatedResult<Invoice>> {
  const params = new URLSearchParams({
    page: String(page),
    pageSize: String(pageSize),
    q: search,
    ...(status ? { status } : {}),
  })

  return apiGet<PaginatedResult<Invoice>>(`/invoices?${params.toString()}`)
}

export async function fetchInvoice(id: string): Promise<InvoiceDetail | null> {
  try {
    return await apiGet<InvoiceDetail>(`/invoices/${id}`)
  } catch (error) {
    if (error instanceof ApiError && error.status === 404) return null
    throw error
  }
}

export function createInvoice(input: InvoiceInput): Promise<InvoiceDetail> {
  return apiPost<InvoiceDetail>('/invoices', input)
}

export function updateInvoice(id: string, input: InvoiceInput): Promise<InvoiceDetail> {
  return apiPatch<InvoiceDetail>(`/invoices/${id}`, input)
}

export async function deleteInvoice(id: string): Promise<void> {
  await apiDelete(`/invoices/${id}`)
}
