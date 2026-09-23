import { useId } from 'react'

export interface ProfileFormValues {
  name: string
  email: string
}

const FIELD_CLASSES =
  'rounded border border-[var(--border)] bg-transparent px-2 py-1 text-sm text-[var(--text)] outline-none focus:border-[var(--accent)]'

// name/email - always editable inline (no separate view/edit toggle, this
// page has nothing else to look at first).
export function ProfileForm({
  values,
  onChange,
  onSubmit,
  submitting,
}: {
  values: ProfileFormValues
  onChange: (values: ProfileFormValues) => void
  onSubmit: () => void
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
      <label htmlFor={`${idPrefix}-name`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
        name
        <input
          id={`${idPrefix}-name`}
          value={values.name}
          onChange={(event) => onChange({ ...values, name: event.target.value })}
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
      <div className="mt-2">
        <button
          type="submit"
          disabled={submitting || values.name === '' || values.email === ''}
          className="rounded border border-[var(--accent)] bg-[var(--accent-dim)] px-3 py-1.5 text-xs text-[var(--accent)] transition-colors disabled:opacity-50"
        >
          {submitting ? 'saving...' : 'save'}
        </button>
      </div>
    </form>
  )
}
