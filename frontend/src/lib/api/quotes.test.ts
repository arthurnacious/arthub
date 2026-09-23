import { afterEach, describe, expect, it, vi } from 'vitest'
import {
  createQuote,
  deleteQuote,
  fetchQuote,
  fetchQuotes,
  updateQuote,
} from './quotes'
import type { Quote, QuoteDetail, QuoteInput } from './quotes'
import { ApiError } from './http'
import * as http from './http'

const QUOTE: Quote = {
  id: '1',
  quoteNumber: 'Q-0001',
  projectId: 'p1',
  projectName: 'Marketing Site',
  clientId: 'c1',
  clientName: 'Nimbus Retail',
  status: 'draft',
  issueDate: null,
  expiryDate: null,
  total: 100,
}

const QUOTE_DETAIL: QuoteDetail = {
  id: '1',
  quoteNumber: 'Q-0001',
  projectId: 'p1',
  projectName: 'Marketing Site',
  clientId: 'c1',
  clientName: 'Nimbus Retail',
  status: 'draft',
  issueDate: null,
  expiryDate: null,
  discountType: null,
  discountValue: 0,
  notes: '',
  subtotal: 100,
  discount: 0,
  total: 100,
  lines: [],
}

const INPUT: QuoteInput = {
  projectId: 'p1',
  status: 'draft',
  issueDate: null,
  expiryDate: null,
  discountType: null,
  discountValue: 0,
  notes: '',
  lineItems: [{ description: 'Design', quantity: 1, unitPrice: 100, discountType: null, discountValue: 0 }],
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('fetchQuotes', () => {
  it('requests the right query string and returns the result as-is', async () => {
    const paginated = { data: [QUOTE], page: 2, pageSize: 10, total: 15 }
    const apiGetSpy = vi.spyOn(http, 'apiGet').mockResolvedValue(paginated)

    const result = await fetchQuotes({ page: 2, pageSize: 10, search: 'nimbus' })

    expect(apiGetSpy).toHaveBeenCalledWith('/quotes?page=2&pageSize=10&q=nimbus')
    expect(result).toEqual(paginated)
  })

  it('includes status in the query string when given', async () => {
    const apiGetSpy = vi
      .spyOn(http, 'apiGet')
      .mockResolvedValue({ data: [], page: 1, pageSize: 10, total: 0 })

    await fetchQuotes({ page: 1, pageSize: 10, search: '', status: 'sent' })

    expect(apiGetSpy).toHaveBeenCalledWith('/quotes?page=1&pageSize=10&q=&status=sent')
  })
})

describe('fetchQuote', () => {
  it('returns the quote detail on success', async () => {
    vi.spyOn(http, 'apiGet').mockResolvedValue(QUOTE_DETAIL)

    expect(await fetchQuote('1')).toEqual(QUOTE_DETAIL)
  })

  it('returns null (not a throw) on a 404', async () => {
    vi.spyOn(http, 'apiGet').mockRejectedValue(new ApiError('Quote not found', 404))

    expect(await fetchQuote('nonexistent')).toBeNull()
  })

  it('rethrows non-404 errors', async () => {
    vi.spyOn(http, 'apiGet').mockRejectedValue(new ApiError('Server error', 500))

    await expect(fetchQuote('1')).rejects.toBeInstanceOf(ApiError)
  })
})

describe('createQuote', () => {
  it('posts the input (including line items) and returns the created quote', async () => {
    const apiPostSpy = vi.spyOn(http, 'apiPost').mockResolvedValue(QUOTE_DETAIL)

    const result = await createQuote(INPUT)

    expect(apiPostSpy).toHaveBeenCalledWith('/quotes', INPUT)
    expect(result).toEqual(QUOTE_DETAIL)
  })
})

describe('updateQuote', () => {
  it('patches the given id with the given fields', async () => {
    const apiPatchSpy = vi.spyOn(http, 'apiPatch').mockResolvedValue(QUOTE_DETAIL)

    const result = await updateQuote('1', INPUT)

    expect(apiPatchSpy).toHaveBeenCalledWith('/quotes/1', INPUT)
    expect(result).toEqual(QUOTE_DETAIL)
  })
})

describe('deleteQuote', () => {
  it('sends a DELETE for the given id', async () => {
    const apiDeleteSpy = vi.spyOn(http, 'apiDelete').mockResolvedValue(undefined)

    await deleteQuote('1')

    expect(apiDeleteSpy).toHaveBeenCalledWith('/quotes/1')
  })
})
