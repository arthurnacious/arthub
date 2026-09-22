import { describe, expect, it } from 'vitest'
import { generateMockActivity } from './mock-activity'

describe('generateMockActivity', () => {
  it('returns one entry per requested day, ending today', () => {
    const days = generateMockActivity(30)
    const today = new Date()
    const expectedLast = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`

    expect(days).toHaveLength(30)
    expect(days[days.length - 1].date).toBe(expectedLast)
  })

  it('returns dates in ascending order with no gaps', () => {
    const days = generateMockActivity(14)

    for (let i = 1; i < days.length; i++) {
      const prev = new Date(days[i - 1].date)
      const curr = new Date(days[i].date)
      const diffDays = (curr.getTime() - prev.getTime()) / (1000 * 60 * 60 * 24)
      expect(diffDays).toBe(1)
    }
  })

  it('only produces non-negative integer counts', () => {
    const days = generateMockActivity(60)

    for (const day of days) {
      expect(Number.isInteger(day.count)).toBe(true)
      expect(day.count).toBeGreaterThanOrEqual(0)
    }
  })
})
