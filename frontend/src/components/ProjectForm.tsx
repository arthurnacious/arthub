import { useId } from 'react'
import { useQuery } from '@tanstack/react-query'
import { fetchClients } from '../lib/api/clients'
import { fetchProjectCategories } from '../lib/api/projects'
import type { ProjectStatus } from '../lib/api/projects'

export interface ProjectFormValues {
  clientId: string
  name: string
  description: string
  category: string
  url: string
  status: ProjectStatus
}

const STATUS_OPTIONS: ProjectStatus[] = ['active', 'completed', 'archived']

const FIELD_CLASSES =
  'rounded border border-[var(--border)] bg-transparent px-2 py-1 text-sm text-[var(--text)] outline-none focus:border-[var(--accent)]'
const BUTTON_CLASSES =
  'rounded border border-[var(--border)] px-3 py-1.5 text-xs text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]'

// Shared client/name/description/category/url/status form - used both by
// the "new project" modal and the project detail page's edit mode. Image
// upload lives on the detail page instead (a project needs to exist
// before images can be attached to it).
export function ProjectForm({
  values,
  onChange,
  onSubmit,
  onCancel,
  submitLabel,
  submitting,
}: {
  values: ProjectFormValues
  onChange: (values: ProjectFormValues) => void
  onSubmit: () => void
  onCancel: () => void
  submitLabel: string
  submitting: boolean
}) {
  const idPrefix = useId()

  // pageSize large enough to cover this app's scale in one request - a
  // dropdown, not a paginated list, so there's no server-side filtering
  // to fall back to here.
  const { data: clients } = useQuery({
    queryKey: ['clients', { page: 1, pageSize: 200, search: '' }],
    queryFn: () => fetchClients({ page: 1, pageSize: 200, search: '' }),
  })

  const { data: categories } = useQuery({
    queryKey: ['project-categories'],
    queryFn: fetchProjectCategories,
  })

  return (
    <form
      onSubmit={(event) => {
        event.preventDefault()
        onSubmit()
      }}
      className="flex flex-col gap-3"
    >
      <label htmlFor={`${idPrefix}-client`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
        client
        <select
          id={`${idPrefix}-client`}
          value={values.clientId}
          onChange={(event) => onChange({ ...values, clientId: event.target.value })}
          className={`${FIELD_CLASSES} bg-[var(--bg)]`}
        >
          <option value="" disabled>
            select a client...
          </option>
          {clients?.data.map((client) => (
            <option key={client.id} value={client.id}>
              {client.company}
            </option>
          ))}
        </select>
      </label>
      <label htmlFor={`${idPrefix}-name`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
        name
        <input
          id={`${idPrefix}-name`}
          value={values.name}
          onChange={(event) => onChange({ ...values, name: event.target.value })}
          className={FIELD_CLASSES}
        />
      </label>
      <label
        htmlFor={`${idPrefix}-description`}
        className="flex flex-col gap-1 text-xs text-[var(--text-dim)]"
      >
        description
        <textarea
          id={`${idPrefix}-description`}
          value={values.description}
          onChange={(event) => onChange({ ...values, description: event.target.value })}
          rows={3}
          className={FIELD_CLASSES}
        />
      </label>
      <label
        htmlFor={`${idPrefix}-category`}
        className="flex flex-col gap-1 text-xs text-[var(--text-dim)]"
      >
        category
        <input
          id={`${idPrefix}-category`}
          list={`${idPrefix}-category-options`}
          value={values.category}
          onChange={(event) => onChange({ ...values, category: event.target.value })}
          placeholder="e.g. web, branding, logo"
          className={FIELD_CLASSES}
        />
        <datalist id={`${idPrefix}-category-options`}>
          {categories?.map((category) => <option key={category} value={category} />)}
        </datalist>
      </label>
      <label htmlFor={`${idPrefix}-url`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
        url <span className="normal-case text-[var(--text-dim)]">(leave blank if not published)</span>
        <input
          id={`${idPrefix}-url`}
          type="url"
          value={values.url}
          onChange={(event) => onChange({ ...values, url: event.target.value })}
          placeholder="https://..."
          className={FIELD_CLASSES}
        />
      </label>
      <label htmlFor={`${idPrefix}-status`} className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
        status
        <select
          id={`${idPrefix}-status`}
          value={values.status}
          onChange={(event) =>
            onChange({ ...values, status: event.target.value as ProjectStatus })
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
          disabled={submitting || values.clientId === ''}
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
