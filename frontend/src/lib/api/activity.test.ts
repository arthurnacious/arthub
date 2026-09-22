import { afterEach, describe, expect, it, vi } from 'vitest'
import { fetchActivity } from './activity'
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
