import { beforeEach, describe, expect, it } from 'vitest'
import {
  __resetClientsStoreForTests,
  deleteClient,
  fetchClient,
  fetchClients,
  updateClient,
} from './clients'
import { MOCK_CLIENTS } from '../mock-clients'

beforeEach(() => {
  __resetClientsStoreForTests()
})

describe('fetchClients', () => {
  it('returns a page-sized slice with pagination metadata', async () => {
    const result = await fetchClients({ page: 1, pageSize: 2, search: '' })

    expect(result.data).toHaveLength(2)
    expect(result.page).toBe(1)
    expect(result.pageSize).toBe(2)
    expect(result.total).toBe(MOCK_CLIENTS.length)
  })

  it('returns a different slice for later pages', async () => {
    const page1 = await fetchClients({ page: 1, pageSize: 2, search: '' })
    const page2 = await fetchClients({ page: 2, pageSize: 2, search: '' })

    expect(page2.data[0].id).not.toBe(page1.data[0].id)
  })

  it('applies the search filter before paginating (total reflects the filtered count)', async () => {
    const result = await fetchClients({ page: 1, pageSize: 10, search: 'nimbus' })

    expect(result.data).toHaveLength(1)
    expect(result.total).toBe(1)
    expect(result.data[0].company).toBe('Nimbus Retail')
  })

  it('returns an empty page past the end of the results, not an error', async () => {
    const result = await fetchClients({ page: 999, pageSize: 10, search: '' })

    expect(result.data).toEqual([])
    expect(result.total).toBe(MOCK_CLIENTS.length)
  })
})

describe('fetchClient', () => {
  it('returns the client with the given id', async () => {
    const client = await fetchClient('1')
    expect(client?.company).toBe('Nimbus Retail')
  })

  it('returns null for an unknown id', async () => {
    expect(await fetchClient('nonexistent')).toBeNull()
  })
})

describe('updateClient', () => {
  it('patches the given fields and returns the updated client', async () => {
    const updated = await updateClient('1', { company: 'Nimbus Retail Group' })

    expect(updated.company).toBe('Nimbus Retail Group')
    expect(updated.id).toBe('1')
  })

  it('persists the change for subsequent fetches', async () => {
    await updateClient('1', { status: 'inactive' })

    const refetched = await fetchClient('1')
    expect(refetched?.status).toBe('inactive')
  })

  it('rejects an unknown id', async () => {
    await expect(updateClient('nonexistent', { company: 'x' })).rejects.toThrow()
  })
})

describe('deleteClient', () => {
  it('removes the client so it no longer appears in fetchClients', async () => {
    await deleteClient('1')

    const result = await fetchClients({ page: 1, pageSize: 100, search: '' })
    expect(result.data.find((c) => c.id === '1')).toBeUndefined()
    expect(result.total).toBe(MOCK_CLIENTS.length - 1)
  })

  it('makes fetchClient return null afterwards', async () => {
    await deleteClient('1')
    expect(await fetchClient('1')).toBeNull()
  })
})
