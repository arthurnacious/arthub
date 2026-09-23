// Shared by Quotes and Invoices - both are "a project, a discount, a list
// of billable lines" with byte-identical math (mirrors
// QuotesController/InvoicesController exactly, see backend/src/Modules/
// {Quotes,Invoices}/*Controller.php) and an identical editing UI (see
// components/LineItemsEditor.tsx). Kept as one shared implementation
// rather than two near-identical copies, since money math is exactly the
// kind of thing that shouldn't be allowed to drift between two call sites.
export type DiscountType = 'percent' | 'fixed' | null

export interface LineItem {
  description: string
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

export function lineTotal(line: LineItem): number {
  const subtotal = line.quantity * line.unitPrice
  const discount = discountAmount(subtotal, line.discountType, line.discountValue)

  return Math.max(0, round(subtotal - discount))
}

export function computeTotals(
  lines: LineItem[],
  overallDiscountType: DiscountType,
  overallDiscountValue: number,
): { subtotal: number; discount: number; total: number } {
  const subtotal = round(lines.reduce((sum, line) => sum + lineTotal(line), 0))
  const discount = round(discountAmount(subtotal, overallDiscountType, overallDiscountValue))
  const total = Math.max(0, round(subtotal - discount))

  return { subtotal, discount, total }
}

function round(value: number): number {
  return Math.round(value * 100) / 100
}
