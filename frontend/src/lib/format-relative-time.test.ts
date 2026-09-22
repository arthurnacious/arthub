import { describe, expect, it } from 'vitest'
import { formatRelativeTime } from './format-relative-time'

const NOW = new Date('2026-09-22T12:00:00Z')

describe('formatRelativeTime', () => {
  it('says "just now" for under a minute', () => {
    expect(formatRelativeTime('2026-09-22T11:59:30Z', NOW)).toBe('just now')
  })

  it('formats minutes', () => {
    expect(formatRelativeTime('2026-09-22T11:55:00Z', NOW)).toBe('5m ago')
  })

  it('formats hours', () => {
    expect(formatRelativeTime('2026-09-22T09:00:00Z', NOW)).toBe('3h ago')
  })

  it('formats days', () => {
    expect(formatRelativeTime('2026-09-19T12:00:00Z', NOW)).toBe('3d ago')
  })
})
