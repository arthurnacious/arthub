import type { DiscountType } from './api/quotes'

// Mirrors QuotesController's math exactly (see
// backend/src/Modules/Quotes/QuotesController.php) so the form can show a
// live total as the user edits - the server recomputes authoritatively on
// save, this is just for instant feedback.
export interface LineInput {
  quantity: number
  unitPrice: number
  discountType: DiscountType
  discountValue: number
}

function discountAmount(base: number, type: DiscountType, value: number): number {
  if (type === 'percent') return base * (value / 100)
  if (type === 'fixed') return value
  return 0
}

export function lineTotal(line: LineInput): number {
  const subtotal = line.quantity * line.unitPrice
  const discount = discountAmount(subtotal, line.discountType, line.discountValue)

  return Math.max(0, round(subtotal - discount))
}

export function quoteTotals(
  lines: LineInput[],
  quoteDiscountType: DiscountType,
  quoteDiscountValue: number,
): { subtotal: number; discount: number; total: number } {
  const subtotal = round(lines.reduce((sum, line) => sum + lineTotal(line), 0))
  const discount = round(discountAmount(subtotal, quoteDiscountType, quoteDiscountValue))
  const total = Math.max(0, round(subtotal - discount))

  return { subtotal, discount, total }
}

function round(value: number): number {
  return Math.round(value * 100) / 100
}
