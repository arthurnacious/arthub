import { ApiError, apiDelete, apiGet, apiPatch, apiPost } from './http'
import type { PaginatedResult, PaginationParams } from './pagination'
import type { StatusTone } from '../../components/StatusBadge'
import type { DiscountType, LineItem } from '../line-items'

export type { DiscountType } from '../line-items'
export type QuoteStatus = 'draft' | 'sent' | 'accepted' | 'rejected' | 'expired'

export const QUOTE_STATUS_TONE: Record<QuoteStatus, StatusTone> = {
  draft: 'neutral',
  sent: 'info',
  accepted: 'success',
  rejected: 'danger',
  expired: 'warning',
}

// Matches QuotesController::present() - see
// backend/src/Modules/Quotes/QuotesController.php.
export interface Quote {
  id: string
  quoteNumber: string
  projectId: string
  projectName: string
  clientId: string
  clientName: string
  status: QuoteStatus
  issueDate: string | null
  expiryDate: string | null
  total: number
}

export interface QuoteLine {
  id: string
  description: string
  quantity: number
  unitPrice: number
  discountType: DiscountType
  discountValue: number
  lineTotal: number
  position: number
}

// Matches QuotesController::presentDetail().
export interface QuoteDetail {
  id: string
  quoteNumber: string
  projectId: string
  projectName: string
  clientId: string
  clientName: string
  status: QuoteStatus
  issueDate: string | null
  expiryDate: string | null
  discountType: DiscountType
  discountValue: number
  notes: string
  subtotal: number
  discount: number
  total: number
  lines: QuoteLine[]
}

// A line item as sent to the backend - no id/lineTotal (the server
// computes those; an update replaces every line rather than diffing).
// Same shape as the shared LineItem type used by the editor UI - kept as
// an alias so call sites read naturally as "a quote's line input".
export type QuoteLineInput = LineItem

export interface QuoteInput {
  projectId: string
  status: QuoteStatus
  issueDate: string | null
  expiryDate: string | null
  discountType: DiscountType
  discountValue: number
  notes: string
  lineItems: QuoteLineInput[]
}

// GET /quotes?page=&pageSize=&q=&status= - server-side pagination,
// search (quote number, project name, or client name) and status filter.
export function fetchQuotes({
  page,
  pageSize,
  search,
  status = '',
}: PaginationParams & { status?: string }): Promise<PaginatedResult<Quote>> {
  const params = new URLSearchParams({
    page: String(page),
    pageSize: String(pageSize),
    q: search,
    ...(status ? { status } : {}),
  })

  return apiGet<PaginatedResult<Quote>>(`/quotes?${params.toString()}`)
}

export async function fetchQuote(id: string): Promise<QuoteDetail | null> {
  try {
    return await apiGet<QuoteDetail>(`/quotes/${id}`)
  } catch (error) {
    if (error instanceof ApiError && error.status === 404) return null
    throw error
  }
}

export function createQuote(input: QuoteInput): Promise<QuoteDetail> {
  return apiPost<QuoteDetail>('/quotes', input)
}

export function updateQuote(id: string, input: QuoteInput): Promise<QuoteDetail> {
  return apiPatch<QuoteDetail>(`/quotes/${id}`, input)
}

export async function deleteQuote(id: string): Promise<void> {
  await apiDelete(`/quotes/${id}`)
}
