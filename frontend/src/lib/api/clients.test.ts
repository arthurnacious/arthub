import { afterEach, describe, expect, it, vi } from 'vitest'
import {
  createClient,
  deleteClient,
  fetchClient,
  fetchClients,
  updateClient,
} from './clients'
import { ApiError } from './http'
import * as http from './http'
import type { Client } from '../mock-clients'

const CLIENT: Client = {
  id: '1',
  company: 'Nimbus Retail',
  contact: 'Dana Whitfield',
  email: 'dana@nimbusretail.test',
  status: 'active',
  projects: 3,
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('fetchClients', () => {
  it('requests the right query string and returns the result as-is', async () => {
    const paginated = { data: [CLIENT], page: 2, pageSize: 5, total: 11 }
    const apiGetSpy = vi.spyOn(http, 'apiGet').mockResolvedValue(paginated)

    const result = await fetchClients({ page: 2, pageSize: 5, search: 'nimbus' })

    expect(apiGetSpy).toHaveBeenCalledWith('/clients?page=2&pageSize=5&q=nimbus')
    expect(result).toEqual(paginated)
  })
})

describe('fetchClient', () => {
  it('returns the client on success', async () => {
    vi.spyOn(http, 'apiGet').mockResolvedValue(CLIENT)

    expect(await fetchClient('1')).toEqual(CLIENT)
  })

  it('returns null (not a throw) on a 404', async () => {
    vi.spyOn(http, 'apiGet').mockRejectedValue(new ApiError('Client not found', 404))

    expect(await fetchClient('nonexistent')).toBeNull()
  })

  it('rethrows non-404 errors', async () => {
    vi.spyOn(http, 'apiGet').mockRejectedValue(new ApiError('Server error', 500))

    await expect(fetchClient('1')).rejects.toBeInstanceOf(ApiError)
  })
})

describe('createClient', () => {
  it('posts the input and returns the created client', async () => {
    const apiPostSpy = vi.spyOn(http, 'apiPost').mockResolvedValue(CLIENT)

    const result = await createClient({
      company: 'Nimbus Retail',
      contact: 'Dana Whitfield',
      email: 'dana@nimbusretail.test',
      status: 'active',
    })

    expect(apiPostSpy).toHaveBeenCalledWith('/clients', {
      company: 'Nimbus Retail',
      contact: 'Dana Whitfield',
      email: 'dana@nimbusretail.test',
      status: 'active',
    })
    expect(result).toEqual(CLIENT)
  })
})

describe('updateClient', () => {
  it('patches the given id with the given fields', async () => {
    const apiPatchSpy = vi.spyOn(http, 'apiPatch').mockResolvedValue(CLIENT)

    const result = await updateClient('1', { company: 'New Name' })

    expect(apiPatchSpy).toHaveBeenCalledWith('/clients/1', { company: 'New Name' })
    expect(result).toEqual(CLIENT)
  })
})

describe('deleteClient', () => {
  it('sends a DELETE for the given id', async () => {
    const apiDeleteSpy = vi.spyOn(http, 'apiDelete').mockResolvedValue(undefined)

    await deleteClient('1')

    expect(apiDeleteSpy).toHaveBeenCalledWith('/clients/1')
  })
})
