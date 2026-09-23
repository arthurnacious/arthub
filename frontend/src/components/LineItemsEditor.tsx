import { lineTotal } from '../lib/line-items'
import type { LineItem } from '../lib/line-items'

const EMPTY_LINE: LineItem = {
  description: '',
  quantity: 1,
  unitPrice: 0,
  discountType: null,
  discountValue: 0,
}

const INPUT_CLASSES =
  'w-full rounded border border-[var(--border)] bg-transparent px-2 py-1 text-sm text-[var(--text)] outline-none focus:border-[var(--accent)]'

function formatCurrency(value: number): string {
  return value.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

// A repeatable table of line items - add/remove rows, each with its own
// optional discount, and a live-computed total per row (see
// lib/line-items.ts, which mirrors the backend's math exactly). Shared by
// QuoteForm and InvoiceForm - both need byte-identical editing behavior.
export function LineItemsEditor({
  lines,
  onChange,
}: {
  lines: LineItem[]
  onChange: (lines: LineItem[]) => void
}) {
  function updateLine(index: number, patch: Partial<LineItem>) {
    onChange(lines.map((line, i) => (i === index ? { ...line, ...patch } : line)))
  }

  function removeLine(index: number) {
    onChange(lines.filter((_, i) => i !== index))
  }

  function addLine() {
    onChange([...lines, { ...EMPTY_LINE }])
  }

  return (
    <div className="flex flex-col gap-2">
      {lines.length === 0 ? (
        <p className="text-xs text-[var(--text-dim)]">-- no line items yet --</p>
      ) : (
        <div className="flex flex-col gap-2">
          {lines.map((line, index) => (
            <div
              key={index}
              className="grid grid-cols-2 gap-2 rounded border border-[var(--border)] p-2 sm:grid-cols-[2fr_1fr_1fr_1fr_1fr_auto]"
            >
              <input
                value={line.description}
                onChange={(event) => updateLine(index, { description: event.target.value })}
                placeholder="description"
                aria-label={`Line ${index + 1} description`}
                className={`${INPUT_CLASSES} col-span-2 sm:col-span-1`}
              />
              <input
                type="number"
                min="0"
                step="any"
                value={line.quantity}
                onChange={(event) => updateLine(index, { quantity: Number(event.target.value) })}
                aria-label={`Line ${index + 1} quantity`}
                className={INPUT_CLASSES}
              />
              <input
                type="number"
                min="0"
                step="any"
                value={line.unitPrice}
                onChange={(event) => updateLine(index, { unitPrice: Number(event.target.value) })}
                aria-label={`Line ${index + 1} unit price`}
                className={INPUT_CLASSES}
              />
              <select
                value={line.discountType ?? ''}
                onChange={(event) =>
                  updateLine(index, {
                    discountType: event.target.value === '' ? null : (event.target.value as 'percent' | 'fixed'),
                  })
                }
                aria-label={`Line ${index + 1} discount type`}
                className={`${INPUT_CLASSES} bg-[var(--bg)]`}
              >
                <option value="">no discount</option>
                <option value="percent">%</option>
                <option value="fixed">$</option>
              </select>
              <input
                type="number"
                min="0"
                step="any"
                value={line.discountValue}
                onChange={(event) => updateLine(index, { discountValue: Number(event.target.value) })}
                disabled={line.discountType === null}
                aria-label={`Line ${index + 1} discount value`}
                className={`${INPUT_CLASSES} disabled:opacity-40`}
              />
              <div className="col-span-2 flex items-center justify-between text-xs text-[var(--text-dim)] sm:col-span-1 sm:flex-col sm:items-end sm:justify-center sm:gap-1">
                <span>{formatCurrency(lineTotal(line))}</span>
                <button
                  type="button"
                  onClick={() => removeLine(index)}
                  aria-label={`Remove line ${index + 1}`}
                  className="text-[var(--text-dim)] transition-colors hover:text-red-400"
                >
                  &#10005;
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
      <button
        type="button"
        onClick={addLine}
        className="self-start rounded border border-[var(--border)] px-2 py-1 text-xs text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]"
      >
        + add line
      </button>
    </div>
  )
}
