import { useId } from 'react'
import { useQuery } from '@tanstack/react-query'
import { fetchProjects } from '../lib/api/projects'
import { LineItemsEditor } from './LineItemsEditor'
import { computeTotals } from '../lib/line-items'
import type { QuoteInput, QuoteStatus } from '../lib/api/quotes'

const STATUS_OPTIONS: QuoteStatus[] = ['draft', 'sent', 'accepted', 'rejected', 'expired']

const FIELD_CLASSES =
  'rounded border border-[var(--border)] bg-transparent px-2 py-1 text-sm text-[var(--text)] outline-none focus:border-[var(--accent)]'
const BUTTON_CLASSES =
  'rounded border border-[var(--border)] px-3 py-1.5 text-xs text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]'

function formatCurrency(value: number): string {
  return value.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

// Shared project/status/dates/discount/notes/line-items form - used both
// by the "new quote" modal and the quote detail page's edit mode. Line
// items live inline here (not a separate step, unlike Projects' images -
// a quote isn't meaningful without them).
export function QuoteForm({
  values,
  onChange,
  onSubmit,
  onCancel,
  submitLabel,
  submitting,
}: {
  values: QuoteInput
  onChange: (values: QuoteInput) => void
  onSubmit: () => void
  onCancel: () => void
  submitLabel: string
  submitting: boolean
}) {
  const idPrefix = useId()

  // pageSize large enough to cover this app's scale in one request - a
  // dropdown, not a paginated list, same as ProjectForm's client picker.
  const { data: projects } = useQuery({
    queryKey: ['projects', { page: 1, pageSize: 200, search: '' }],
    queryFn: () => fetchProjects({ page: 1, pageSize: 200, search: '' }),
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

      <div className="grid grid-cols-2 gap-3">
        <label htmlFor={`${idPrefix}-status`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
          status
          <select
            id={`${idPrefix}-status`}
            value={values.status}
            onChange={(event) => onChange({ ...values, status: event.target.value as QuoteStatus })}
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
          <label htmlFor={`${idPrefix}-expiry`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
            expires
            <input
              id={`${idPrefix}-expiry`}
              type="date"
              value={values.expiryDate ?? ''}
              onChange={(event) => onChange({ ...values, expiryDate: event.target.value || null })}
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
