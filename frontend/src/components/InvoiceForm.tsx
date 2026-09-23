import { useId } from 'react'
import { useQuery } from '@tanstack/react-query'
import { fetchProjects } from '../lib/api/projects'
import { fetchQuotes } from '../lib/api/quotes'
import { LineItemsEditor } from './LineItemsEditor'
import { computeTotals } from '../lib/line-items'
import type { InvoiceInput, InvoiceStatus } from '../lib/api/invoices'

const STATUS_OPTIONS: InvoiceStatus[] = ['draft', 'sent', 'paid', 'overdue', 'cancelled']

const FIELD_CLASSES =
  'rounded border border-[var(--border)] bg-transparent px-2 py-1 text-sm text-[var(--text)] outline-none focus:border-[var(--accent)]'
const BUTTON_CLASSES =
  'rounded border border-[var(--border)] px-3 py-1.5 text-xs text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]'

function formatCurrency(value: number): string {
  return value.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

// Shared project/quote/status/dates/discount/notes/line-items form - used
// both by the "new invoice" modal and the invoice detail page's edit
// mode. paidAt isn't a field here: the backend derives it automatically
// from the status transition (set the instant status becomes 'paid',
// cleared when it moves away) rather than accepting it as input.
export function InvoiceForm({
  values,
  onChange,
  onSubmit,
  onCancel,
  submitLabel,
  submitting,
}: {
  values: InvoiceInput
  onChange: (values: InvoiceInput) => void
  onSubmit: () => void
  onCancel: () => void
  submitLabel: string
  submitting: boolean
}) {
  const idPrefix = useId()

  // pageSize large enough to cover this app's scale in one request - a
  // dropdown, not a paginated list, same as ProjectForm/QuoteForm.
  const { data: projects } = useQuery({
    queryKey: ['projects', { page: 1, pageSize: 200, search: '' }],
    queryFn: () => fetchProjects({ page: 1, pageSize: 200, search: '' }),
  })
  const { data: quotes } = useQuery({
    queryKey: ['quotes', { page: 1, pageSize: 200, search: '', status: '' }],
    queryFn: () => fetchQuotes({ page: 1, pageSize: 200, search: '' }),
  })

  const totals = computeTotals(values.lineItems, values.discountType, values.discountValue)

  return (
    <form
      onSubmit={(event) => {
        event.preventDefault()
        onSubmit()
      }}
      className="flex flex-col gap-3"
    >
      <div className="grid grid-cols-2 gap-3">
        <label htmlFor={`${idPrefix}-project`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
          project
          <select
            id={`${idPrefix}-project`}
            value={values.projectId}
            onChange={(event) => onChange({ ...values, projectId: event.target.value })}
            className={`${FIELD_CLASSES} bg-[var(--bg)]`}
          >
            <option value="" disabled>
              select a project...
            </option>
            {projects?.data.map((project) => (
              <option key={project.id} value={project.id}>
                {project.name} ({project.clientName})
              </option>
            ))}
          </select>
        </label>
        <label htmlFor={`${idPrefix}-quote`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
          from quote
          <select
            id={`${idPrefix}-quote`}
            value={values.quoteId ?? ''}
            onChange={(event) => onChange({ ...values, quoteId: event.target.value || null })}
            className={`${FIELD_CLASSES} bg-[var(--bg)]`}
          >
            <option value="">none</option>
            {quotes?.data.map((quote) => (
              <option key={quote.id} value={quote.id}>
                {quote.quoteNumber} ({quote.projectName})
              </option>
            ))}
          </select>
        </label>
      </div>

      <div className="grid grid-cols-2 gap-3">
        <label htmlFor={`${idPrefix}-status`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
          status
          <select
            id={`${idPrefix}-status`}
            value={values.status}
            onChange={(event) => onChange({ ...values, status: event.target.value as InvoiceStatus })}
            className={`${FIELD_CLASSES} bg-[var(--bg)]`}
          >
            {STATUS_OPTIONS.map((status) => (
              <option key={status} value={status}>
                {status}
              </option>
            ))}
          </select>
        </label>
        <div className="grid grid-cols-2 gap-2">
          <label htmlFor={`${idPrefix}-issue`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
            issued
            <input
              id={`${idPrefix}-issue`}
              type="date"
              value={values.issueDate ?? ''}
              onChange={(event) => onChange({ ...values, issueDate: event.target.value || null })}
              className={FIELD_CLASSES}
            />
          </label>
          <label htmlFor={`${idPrefix}-due`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
            due
            <input
              id={`${idPrefix}-due`}
              type="date"
              value={values.dueDate ?? ''}
              onChange={(event) => onChange({ ...values, dueDate: event.target.value || null })}
              className={FIELD_CLASSES}
            />
          </label>
        </div>
      </div>

      <div>
        <span className="text-xs text-[var(--text-dim)]">line items</span>
        <div className="mt-1">
          <LineItemsEditor
            lines={values.lineItems}
            onChange={(lineItems) => onChange({ ...values, lineItems })}
          />
        </div>
      </div>

      <div className="grid grid-cols-[1fr_1fr_auto] items-end gap-2">
        <label htmlFor={`${idPrefix}-discount-type`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
          overall discount
          <select
            id={`${idPrefix}-discount-type`}
            value={values.discountType ?? ''}
            onChange={(event) =>
              onChange({
                ...values,
                discountType: event.target.value === '' ? null : (event.target.value as 'percent' | 'fixed'),
              })
            }
            className={`${FIELD_CLASSES} bg-[var(--bg)]`}
          >
            <option value="">no discount</option>
            <option value="percent">%</option>
            <option value="fixed">$</option>
          </select>
        </label>
        <label htmlFor={`${idPrefix}-discount-value`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
          amount
          <input
            id={`${idPrefix}-discount-value`}
            type="number"
            min="0"
            step="any"
            value={values.discountValue}
            onChange={(event) => onChange({ ...values, discountValue: Number(event.target.value) })}
            disabled={values.discountType === null}
            className={`${FIELD_CLASSES} disabled:opacity-40`}
          />
        </label>
        <div className="pb-1 text-right text-xs text-[var(--text-dim)]">
          <div>subtotal: {formatCurrency(totals.subtotal)}</div>
          <div className="font-bold text-[var(--text)]">total: {formatCurrency(totals.total)}</div>
        </div>
      </div>

      <label htmlFor={`${idPrefix}-notes`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
        notes
        <textarea
          id={`${idPrefix}-notes`}
          value={values.notes}
          onChange={(event) => onChange({ ...values, notes: event.target.value })}
          rows={2}
          className={FIELD_CLASSES}
        />
      </label>

      {values.status === 'paid' && (
        <p className="text-xs text-[var(--text-dim)]">
          // marking this paid stamps today's date automatically
        </p>
      )}

      <div className="mt-2 flex gap-2">
        <button
          type="submit"
          disabled={submitting || values.projectId === ''}
          className="rounded border border-[var(--accent)] bg-[var(--accent-dim)] px-3 py-1.5 text-xs text-[var(--accent)] transition-colors disabled:opacity-50"
        >
          {submitting ? 'saving...' : submitLabel}
        </button>
        <button type="button" onClick={onCancel} className={BUTTON_CLASSES}>
          cancel
        </button>
      </div>
    </form>
  )
}
