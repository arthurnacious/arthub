import { afterEach, describe, expect, it, vi } from 'vitest'
import { fetchActivity, fetchActivityLog } from './activity'
import * as http from './http'
import type { ActivityEntry } from './activity'

const ENTRY: ActivityEntry = {
  id: '1',
  user_id: 'u1',
  action: 'client.created',
  description: 'Created client Nimbus Retail',
  subject_type: 'client',
  subject_id: 'c1',
  metadata: null,
  created_at: '2026-09-22T12:00:00Z',
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('fetchActivity', () => {
  it('GETs /activity-log and unwraps the data array', async () => {
    const apiGetSpy = vi.spyOn(http, 'apiGet').mockResolvedValue({ data: [ENTRY] })

    const result = await fetchActivity()

    expect(apiGetSpy).toHaveBeenCalledWith('/activity-log')
    expect(result).toEqual([ENTRY])
  })
})

describe('fetchActivityLog', () => {
  it('requests the right query string and returns the paginated result as-is', async () => {
    const paginated = { data: [ENTRY], page: 2, pageSize: 5, total: 11 }
    const apiGetSpy = vi.spyOn(http, 'apiGet').mockResolvedValue(paginated)

    const result = await fetchActivityLog({ page: 2, pageSize: 5, search: 'nimbus' })

    expect(apiGetSpy).toHaveBeenCalledWith('/activity-log?page=2&pageSize=5&q=nimbus')
    expect(result).toEqual(paginated)
  })
})
