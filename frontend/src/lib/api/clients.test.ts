import { describe, expect, it } from 'vitest'
import { fetchClients } from './clients'
import { MOCK_CLIENTS } from '../mock-clients'

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
