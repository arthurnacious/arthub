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

// Stand-in for a real `GET /clients?page=&pageSize=&q=` call - same async,
// paginated shape a real fetch() would return, so swapping this out for
// one (once the endpoint and an auth flow to call it with both exist) is a
// one-line change in the query that calls it (src/routes/clients.tsx).
export async function fetchClients({
  page,
  pageSize,
  search,
}: FetchClientsParams): Promise<PaginatedResult<Client>> {
  await new Promise((resolve) => setTimeout(resolve, 250))

  const filtered = filterClients(MOCK_CLIENTS, search)
  const start = (page - 1) * pageSize
  const data = filtered.slice(start, start + pageSize)

  return { data, page, pageSize, total: filtered.length }
}
