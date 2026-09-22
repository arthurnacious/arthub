import { describe, expect, it } from 'vitest'
import { filterClients } from './filter-clients'
import type { Client } from './mock-clients'

const CLIENTS: Client[] = [
  { id: '1', company: 'Nimbus Retail', contact: 'Dana Whitfield', email: 'dana@nimbusretail.test', status: 'active', projects: 3 },
  { id: '2', company: 'Harbour Digital', contact: 'Priya Anand', email: 'priya@harbourdigital.test', status: 'active', projects: 1 },
]

describe('filterClients', () => {
  it('returns everything for a blank query', () => {
    expect(filterClients(CLIENTS, '')).toHaveLength(2)
    expect(filterClients(CLIENTS, '   ')).toHaveLength(2)
  })

  it('matches on company name, case-insensitively', () => {
    const result = filterClients(CLIENTS, 'nimbus')
    expect(result).toHaveLength(1)
    expect(result[0].company).toBe('Nimbus Retail')
  })

  it('matches on contact name', () => {
    const result = filterClients(CLIENTS, 'priya')
    expect(result).toHaveLength(1)
    expect(result[0].contact).toBe('Priya Anand')
  })

  it('matches on email', () => {
    const result = filterClients(CLIENTS, 'harbourdigital')
    expect(result).toHaveLength(1)
    expect(result[0].company).toBe('Harbour Digital')
  })

  it('returns an empty array when nothing matches', () => {
    expect(filterClients(CLIENTS, 'nonexistent')).toEqual([])
  })
})
