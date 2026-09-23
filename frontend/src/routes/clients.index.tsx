import { useEffect, useState } from 'react'
import { createFileRoute } from '@tanstack/react-router'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Panel } from '../components/Panel'
import { ClientsTable } from '../components/ClientsTable'
import { Pagination } from '../components/Pagination'
import { Modal } from '../components/Modal'
import { ClientForm } from '../components/ClientForm'
import type { ClientFormValues } from '../components/ClientForm'
import { fetchClients, deleteClient, createClient } from '../lib/api/clients'
import { useDebouncedValue } from '../hooks/use-debounced-value'
import { useConfirm } from '../hooks/use-confirm'
import type { Client } from '../lib/mock-clients'

const EMPTY_CLIENT: ClientFormValues = {
  company: '',
  contact: '',
  email: '',
  status: 'prospect',
}

const PAGE_SIZE = 5

interface ClientsSearch {
  page: number
  q: string
}

export const Route = createFileRoute('/clients/')({
  validateSearch: (search: Record<string, unknown>): ClientsSearch => ({
    page: Number(search.page) > 0 ? Math.floor(Number(search.page)) : 1,
    q: typeof search.q === 'string' ? search.q : '',
  }),
  component: ClientsPage,
})

function ClientsPage() {
  const { page, q } = Route.useSearch()
  const navigate = Route.useNavigate()
  const queryClient = useQueryClient()
  const { confirm, dialog } = useConfirm()

  // Local, immediately-responsive input state; the URL (and therefore the
  // server query below) only updates once typing settles - see
  // useDebouncedValue.
  const [searchInput, setSearchInput] = useState(q)
  const debouncedSearch = useDebouncedValue(searchInput, 300)

  useEffect(() => {
    if (debouncedSearch === q) return
    navigate({
      search: (prev) => ({ ...prev, q: debouncedSearch, page: 1 }),
      replace: true,
    })
    // Only the debounced value should drive this - navigating shouldn't
    // re-run it, and `q`/`navigate` are stable enough here that including
    // them would just cause redundant re-runs.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debouncedSearch])

  // page/q live in the URL and drive the query key, so each page or search
  // change is its own server request - not an in-memory filter over
  // everything already fetched.
  const { data, isPending } = useQuery({
    queryKey: ['clients', { page, pageSize: PAGE_SIZE, search: q }],
    queryFn: () => fetchClients({ page, pageSize: PAGE_SIZE, search: q }),
    placeholderData: (previousData) => previousData,
  })

  const deleteMutation = useMutation({
    mutationFn: (client: Client) => deleteClient(client.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['clients'] })
    },
  })

  async function handleDelete(client: Client) {
    const confirmed = await confirm({
      title: 'delete client',
      message: `Delete ${client.company}? This can't be undone.`,
      confirmLabel: 'delete',
      tone: 'danger',
    })
    if (!confirmed) return
    deleteMutation.mutate(client)
  }

  const [newClientOpen, setNewClientOpen] = useState(false)
  const [newClientForm, setNewClientForm] = useState<ClientFormValues>(EMPTY_CLIENT)

  const createMutation = useMutation({
    mutationFn: (values: ClientFormValues) => createClient(values),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['clients'] })
      setNewClientOpen(false)
      setNewClientForm(EMPTY_CLIENT)
    },
  })

  return (
    <div>
      <div className="flex items-center justify-between">
        <h1 className="glow-text text-lg font-bold text-[var(--accent)]">CLIENTS</h1>
        <button
          type="button"
          onClick={() => setNewClientOpen(true)}
          className="rounded border border-[var(--accent)] bg-[var(--accent-dim)] px-3 py-1.5 text-xs text-[var(--accent)] transition-colors hover:bg-[var(--accent)] hover:text-[var(--bg)]"
        >
          + new
        </button>
      </div>
      <p className="mt-2 text-sm text-[var(--text-dim)]">
        // server-paginated - showing mock data until a real /clients endpoint exists
      </p>
      <div className="mt-6">
        <Panel title="clients">
          <input
            type="text"
            value={searchInput}
            onChange={(event) => setSearchInput(event.target.value)}
            placeholder="search company, contact or email..."
            className="mb-4 w-full rounded border border-[var(--border)] bg-transparent px-3 py-1.5 text-sm text-[var(--text)] outline-none placeholder:text-[var(--text-dim)] focus:border-[var(--accent)]"
          />
          {isPending ? (
            <p className="text-sm text-[var(--text-dim)]">loading...</p>
          ) : (
            <>
              <ClientsTable
                clients={data?.data ?? []}
                onView={(client) =>
                  navigate({ to: '/clients/$clientId', params: { clientId: client.id } })
                }
                onUpdate={(client) =>
                  navigate({
                    to: '/clients/$clientId',
                    params: { clientId: client.id },
                    search: { edit: true },
                  })
                }
                onDelete={handleDelete}
              />
              <div className="mt-4">
                <Pagination
                  page={page}
                  pageSize={PAGE_SIZE}
                  total={data?.total ?? 0}
                  onPageChange={(nextPage) =>
                    navigate({ search: (prev) => ({ ...prev, page: nextPage }) })
                  }
                />
              </div>
            </>
          )}
        </Panel>
      </div>
      {dialog}
      {newClientOpen && (
        <Modal title="new client" onClose={() => setNewClientOpen(false)}>
          <ClientForm
            values={newClientForm}
            onChange={setNewClientForm}
            onSubmit={() => createMutation.mutate(newClientForm)}
            onCancel={() => setNewClientOpen(false)}
            submitLabel="create"
            submitting={createMutation.isPending}
          />
        </Modal>
      )}
    </div>
  )
}
