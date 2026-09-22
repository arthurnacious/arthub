import { describe, expect, it } from 'vitest'
import { isVimMoveKey, moveSelection } from './vim-navigation'

describe('isVimMoveKey', () => {
  it('recognizes j/k/g/G and the arrow keys', () => {
    for (const key of ['j', 'k', 'g', 'G', 'ArrowDown', 'ArrowUp']) {
      expect(isVimMoveKey(key)).toBe(true)
    }
  })

  it('rejects everything else', () => {
    for (const key of ['l', 'Enter', 'h', 'a', ' ']) {
      expect(isVimMoveKey(key)).toBe(false)
    }
  })
})

describe('moveSelection', () => {
  it('j/ArrowDown moves to the next index', () => {
    expect(moveSelection(0, 'j', 6)).toBe(1)
    expect(moveSelection(2, 'ArrowDown', 6)).toBe(3)
  })

  it('j clamps at the last index', () => {
    expect(moveSelection(5, 'j', 6)).toBe(5)
  })

  it('k/ArrowUp moves to the previous index', () => {
    expect(moveSelection(3, 'k', 6)).toBe(2)
    expect(moveSelection(3, 'ArrowUp', 6)).toBe(2)
  })

  it('k clamps at 0', () => {
    expect(moveSelection(0, 'k', 6)).toBe(0)
  })

  it('g jumps to the first index', () => {
    expect(moveSelection(4, 'g', 6)).toBe(0)
  })

  it('G jumps to the last index', () => {
    expect(moveSelection(1, 'G', 6)).toBe(5)
  })

  it('is a no-op when there is nothing to select', () => {
    expect(moveSelection(0, 'j', 0)).toBe(0)
  })
})
