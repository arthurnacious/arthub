import { useId } from 'react'
import type { ClientStatus } from '../lib/mock-clients'

export interface ClientFormValues {
  company: string
  contact: string
  email: string
  status: ClientStatus
}

const STATUS_OPTIONS: ClientStatus[] = ['active', 'prospect', 'inactive']

const FIELD_CLASSES =
  'rounded border border-[var(--border)] bg-transparent px-2 py-1 text-sm text-[var(--text)] outline-none focus:border-[var(--accent)]'
const BUTTON_CLASSES =
  'rounded border border-[var(--border)] px-3 py-1.5 text-xs text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]'

// Shared company/contact/email/status form - used both by the "new
// client" modal and the client detail page's edit mode.
export function ClientForm({
  values,
  onChange,
  onSubmit,
  onCancel,
  submitLabel,
  submitting,
}: {
  values: ClientFormValues
  onChange: (values: ClientFormValues) => void
  onSubmit: () => void
  onCancel: () => void
  submitLabel: string
  submitting: boolean
}) {
  const idPrefix = useId()

  return (
    <form
      onSubmit={(event) => {
        event.preventDefault()
        onSubmit()
      }}
      className="flex flex-col gap-3"
    >
      <label htmlFor={`${idPrefix}-company`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
        company
        <input
          id={`${idPrefix}-company`}
          value={values.company}
          onChange={(event) => onChange({ ...values, company: event.target.value })}
          className={FIELD_CLASSES}
        />
      </label>
      <label htmlFor={`${idPrefix}-contact`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
        contact
        <input
          id={`${idPrefix}-contact`}
          value={values.contact}
          onChange={(event) => onChange({ ...values, contact: event.target.value })}
          className={FIELD_CLASSES}
        />
      </label>
      <label htmlFor={`${idPrefix}-email`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
        email
        <input
          id={`${idPrefix}-email`}
          type="email"
          value={values.email}
          onChange={(event) => onChange({ ...values, email: event.target.value })}
          className={FIELD_CLASSES}
        />
      </label>
      <label htmlFor={`${idPrefix}-status`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
        status
        <select
          id={`${idPrefix}-status`}
          value={values.status}
          onChange={(event) =>
            onChange({ ...values, status: event.target.value as ClientStatus })
          }
          className={`${FIELD_CLASSES} bg-[var(--bg)]`}
        >
          {STATUS_OPTIONS.map((status) => (
            <option key={status} value={status}>
              {status}
            </option>
          ))}
        </select>
      </label>
      <div className="mt-2 flex gap-2">
        <button
          type="submit"
          disabled={submitting}
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
