import { describe, expect, it } from 'vitest'
import { lineTotal, quoteTotals } from './quote-totals'

describe('lineTotal', () => {
  it('multiplies quantity by unit price with no discount', () => {
    expect(lineTotal({ quantity: 2, unitPrice: 100, discountType: null, discountValue: 0 })).toBe(200)
  })

  it('applies a percent discount', () => {
    expect(lineTotal({ quantity: 2, unitPrice: 100, discountType: 'percent', discountValue: 10 })).toBe(180)
  })

  it('applies a fixed discount', () => {
    expect(lineTotal({ quantity: 1, unitPrice: 50, discountType: 'fixed', discountValue: 5 })).toBe(45)
  })

  it('never goes below zero', () => {
    expect(lineTotal({ quantity: 1, unitPrice: 10, discountType: 'fixed', discountValue: 100 })).toBe(0)
  })
})

describe('quoteTotals', () => {
  it('sums line totals, then applies the quote-level discount on top', () => {
    const lines = [
      { quantity: 2, unitPrice: 100, discountType: 'percent' as const, discountValue: 10 }, // 180
      { quantity: 1, unitPrice: 50, discountType: 'fixed' as const, discountValue: 5 }, // 45
    ]

    const totals = quoteTotals(lines, 'fixed', 50)

    expect(totals).toEqual({ subtotal: 225, discount: 50, total: 175 })
  })

  it('returns zeros for no line items', () => {
    expect(quoteTotals([], null, 0)).toEqual({ subtotal: 0, discount: 0, total: 0 })
  })

  it('never lets the total go below zero', () => {
    const lines = [{ quantity: 1, unitPrice: 10, discountType: null, discountValue: 0 }]

    expect(quoteTotals(lines, 'fixed', 100).total).toBe(0)
  })
})
