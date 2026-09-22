import { describe, expect, it } from 'vitest'
import { activityLink } from './activity-link'

describe('activityLink', () => {
  it('links a client subject to its detail page', () => {
    expect(activityLink({ subject_type: 'client', subject_id: 'c1' })).toEqual({
      to: '/clients/$clientId',
      params: { clientId: 'c1' },
    })
  })

  it('returns null for an unrecognized subject type', () => {
    expect(activityLink({ subject_type: 'invoice', subject_id: 'i1' })).toBeNull()
  })

  it('returns null when there is no subject at all (e.g. a login event)', () => {
    expect(activityLink({ subject_type: null, subject_id: null })).toBeNull()
  })

  it('returns null for a client subject_type with no id', () => {
    expect(activityLink({ subject_type: 'client', subject_id: null })).toBeNull()
  })
})
