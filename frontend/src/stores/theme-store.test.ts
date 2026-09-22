import { describe, expect, it } from 'vitest'
import { createThemeStore, THEME_COLORS } from './theme-store'

function fakeStorage() {
  const data = new Map<string, string>()
  return {
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => {
      data.set(key, value)
    },
  }
}

describe('theme store', () => {
  it('defaults to green when nothing is stored', () => {
    const { store } = createThemeStore(fakeStorage())

    expect(store.state).toBe('green')
  })

  it('reads the previously stored color on creation', () => {
    const storage = fakeStorage()
    storage.setItem('arthub-theme-color', 'purple')

    const { store } = createThemeStore(storage)

    expect(store.state).toBe('purple')
  })

  it('falls back to green for a corrupted/unknown stored value', () => {
    const storage = fakeStorage()
    storage.setItem('arthub-theme-color', 'not-a-real-color')

    const { store } = createThemeStore(storage)

    expect(store.state).toBe('green')
  })

  it('setColor updates the store and persists the choice', () => {
    const storage = fakeStorage()
    const { store, setColor } = createThemeStore(storage)

    setColor('red')

    expect(store.state).toBe('red')
    expect(storage.getItem('arthub-theme-color')).toBe('red')
  })

  it('cycleColor advances green -> purple -> orange -> red -> green', () => {
    const { store, cycleColor } = createThemeStore(fakeStorage())

    const seen = [store.state]
    for (let i = 0; i < THEME_COLORS.length; i++) {
      cycleColor()
      seen.push(store.state)
    }

    expect(seen).toEqual(['green', 'purple', 'orange', 'red', 'green'])
  })

  it('works with no storage at all (SSR-safe)', () => {
    const { store, cycleColor } = createThemeStore(undefined)

    expect(store.state).toBe('green')
    expect(() => cycleColor()).not.toThrow()
    expect(store.state).toBe('purple')
  })
})
