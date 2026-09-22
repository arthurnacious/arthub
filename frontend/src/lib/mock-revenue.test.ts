import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { generateMockRevenue } from './mock-revenue'

describe('generateMockRevenue', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-09-22'))
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('returns a rolling 12-month window ending at the current month', () => {
    const months = generateMockRevenue()

    expect(months).toHaveLength(12)
    expect(months[0]).toMatchObject({ label: 'oct', group: '2025' })
    expect(months[months.length - 1]).toMatchObject({ label: 'sep', group: '2026' })
  })

  it('groups months by the year they actually fall in', () => {
    const months = generateMockRevenue()

    // Oct, Nov, Dec belong to the earlier year; Jan-Sep to the current one.
    expect(months.slice(0, 3).every((m) => m.group === '2025')).toBe(true)
    expect(months.slice(3).every((m) => m.group === '2026')).toBe(true)
  })

  it('gives every month a positive mock value', () => {
    const months = generateMockRevenue()

    for (const month of months) {
      expect(month.value).toBeGreaterThan(0)
    }
  })
})
