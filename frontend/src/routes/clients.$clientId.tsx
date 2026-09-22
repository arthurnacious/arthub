import { useEffect, useState } from 'react'
import { Link, createFileRoute } from '@tanstack/react-router'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Panel } from '../components/Panel'
import { StatusBadge } from '../components/StatusBadge'
import { ClientForm } from '../components/ClientForm'
import { fetchClient, updateClient } from '../lib/api/clients'
import { CLIENT_STATUS_TONE } from '../lib/mock-clients'
import type { Client } from '../lib/mock-clients'

interface ClientDetailSearch {
  edit: boolean
}

export const Route = createFileRoute('/clients/$clientId')({
  validateSearch: (search: Record<string, unknown>): ClientDetailSearch => ({
    edit: search.edit === true || search.edit === 'true',
  }),
  component: ClientDetailPage,
})

const BUTTON_CLASSES =
  'rounded border border-[var(--border)] px-3 py-1.5 text-xs text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]'

function ClientDetailPage() {
  const { clientId } = Route.useParams()
  const { edit: editFromUrl } = Route.useSearch()
  const queryClient = useQueryClient()

  const { data: client, isLoading } = useQuery({
    queryKey: ['client', clientId],
    queryFn: () => fetchClient(clientId),
  })

  const [editing, setEditing] = useState(editFromUrl)
  const [form, setForm] = useState<Client | null>(null)

  useEffect(() => {
    if (client) setForm(client)
  }, [client])

  const mutation = useMutation({
    mutationFn: (patch: Partial<Omit<Client, 'id'>>) => updateClient(clientId, patch),
    onSuccess: (updated) => {
      queryClient.setQueryData(['client', clientId], updated)
      queryClient.invalidateQueries({ queryKey: ['clients'] })
      setEditing(false)
    },
  })

  if (isLoading) {
    return <p className="text-sm text-[var(--text-dim)]">loading...</p>
  }

  if (!client || !form) {
    return (
      <div>
        <h1 className="glow-text text-lg font-bold text-[var(--accent)]">CLIENT NOT FOUND</h1>
        <Link
          to="/clients"
          search={{ page: 1, q: '' }}
          className="mt-4 inline-block text-sm text-[var(--accent)] hover:underline"
        >
          &larr; back to clients
        </Link>
      </div>
    )
  }

  return (
    <div>
      <div className="flex items-center justify-between">
        <h1 className="glow-text text-lg font-bold text-[var(--accent)]">{client.company}</h1>
        <Link
          to="/clients"
          search={{ page: 1, q: '' }}
          className="text-xs text-[var(--text-dim)] hover:text-[var(--accent)]"
        >
          &larr; back to clients
        </Link>
      </div>
      <div className="mt-6 max-w-md">
        <Panel title={editing ? 'edit client' : 'client'}>
          {editing ? (
            <ClientForm
              values={form}
              onChange={setForm}
              onSubmit={() =>
                mutation.mutate({
                  company: form.company,
                  contact: form.contact,
                  email: form.email,
                  status: form.status,
                })
              }
              onCancel={() => {
                setEditing(false)
                setForm(client)
              }}
              submitLabel="save"
              submitting={mutation.isPending}
            />
          ) : (
            <div className="flex flex-col gap-3 text-sm">
              <div>
                <span className="text-[var(--text-dim)]">contact: </span>
                {client.contact}
              </div>
              <div>
                <span className="text-[var(--text-dim)]">email: </span>
                {client.email}
              </div>
              <div>
                <span className="text-[var(--text-dim)]">status: </span>
                <StatusBadge label={client.status} tone={CLIENT_STATUS_TONE[client.status]} />
              </div>
              <div>
                <span className="text-[var(--text-dim)]">projects: </span>
                {client.projects}
              </div>
              <button
                type="button"
                onClick={() => setEditing(true)}
                className={`${BUTTON_CLASSES} mt-2 self-start`}
              >
                edit
              </button>
            </div>
          )}
        </Panel>
      </div>
    </div>
  )
}
