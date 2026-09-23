import { describe, expect, it } from 'vitest'
import { generateAsciiArt } from './ascii-art'

describe('generateAsciiArt', () => {
  it('is deterministic for the same seed', () => {
    expect(generateAsciiArt('project-1')).toBe(generateAsciiArt('project-1'))
  })

  it('differs for different seeds', () => {
    expect(generateAsciiArt('project-1')).not.toBe(generateAsciiArt('project-2'))
  })

  it('produces the requested number of rows and columns', () => {
    const art = generateAsciiArt('project-1', 10, 4)
    const lines = art.split('\n')

    expect(lines).toHaveLength(4)
    for (const line of lines) {
      expect(line).toHaveLength(10)
    }
  })
})
