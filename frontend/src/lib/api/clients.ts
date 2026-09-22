import { ApiError, apiDelete, apiGet, apiPatch, apiPost } from './http'
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

// GET /clients?page=&pageSize=&q= - server-side pagination and search, see
// backend/src/Modules/Clients/ClientsController.php.
export function fetchClients({
  page,
  pageSize,
  search,
}: FetchClientsParams): Promise<PaginatedResult<Client>> {
  const params = new URLSearchParams({
    page: String(page),
    pageSize: String(pageSize),
    q: search,
  })

  return apiGet<PaginatedResult<Client>>(`/clients?${params.toString()}`)
}

export async function fetchClient(id: string): Promise<Client | null> {
  try {
    return await apiGet<Client>(`/clients/${id}`)
  } catch (error) {
    if (error instanceof ApiError && error.status === 404) return null
    throw error
  }
}

export function createClient(input: Omit<Client, 'id' | 'projects'>): Promise<Client> {
  return apiPost<Client>('/clients', input)
}

export function updateClient(
  id: string,
  patch: Partial<Omit<Client, 'id'>>,
): Promise<Client> {
  return apiPatch<Client>(`/clients/${id}`, patch)
}

export async function deleteClient(id: string): Promise<void> {
  await apiDelete(`/clients/${id}`)
}
