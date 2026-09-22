import { MOCK_CLIENTS } from '../mock-clients'
import { filterClients } from '../filter-clients'
import type { Client } from '../mock-clients'

export interface PaginatedResult<T> {
  data: T[]
  page: number
  pageSize: number
  total: number
}

export interface FetchClientsParams {
  page: number
  pageSize: number
  search: string
}

// In-memory mock "database", standing in until a real backend + auth flow
// exist to call it with. Mutated by update/delete below, so changes are
// visible to subsequent fetches within the session (reset on reload).
let clientsStore: Client[] = [...MOCK_CLIENTS]

// Test-only: puts the store back to its seed data between tests.
export function __resetClientsStoreForTests(): void {
  clientsStore = [...MOCK_CLIENTS]
}

function networkDelay(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 250))
}

// Stand-in for a real `GET /clients?page=&pageSize=&q=` call - same async,
// paginated shape a real fetch() would return, so swapping this out for
// one (once the endpoint and an auth flow to call it with both exist) is a
// one-line change in the query that calls it (src/routes/clients.tsx).
export async function fetchClients({
  page,
  pageSize,
  search,
}: FetchClientsParams): Promise<PaginatedResult<Client>> {
  await networkDelay()

  const filtered = filterClients(clientsStore, search)
  const start = (page - 1) * pageSize
  const data = filtered.slice(start, start + pageSize)

  return { data, page, pageSize, total: filtered.length }
}

// Stand-in for `GET /clients/:id`.
export async function fetchClient(id: string): Promise<Client | null> {
  await networkDelay()
  return clientsStore.find((client) => client.id === id) ?? null
}

// Stand-in for `PATCH /clients/:id`.
export async function updateClient(
  id: string,
  patch: Partial<Omit<Client, 'id'>>,
): Promise<Client> {
  await networkDelay()

  const index = clientsStore.findIndex((client) => client.id === id)
  if (index === -1) throw new Error(`No client with id ${id}`)

  const updated = { ...clientsStore[index], ...patch }
  clientsStore = [
    ...clientsStore.slice(0, index),
    updated,
    ...clientsStore.slice(index + 1),
  ]
  return updated
}

// Stand-in for `DELETE /clients/:id`.
export async function deleteClient(id: string): Promise<void> {
  await networkDelay()
  clientsStore = clientsStore.filter((client) => client.id !== id)
}
