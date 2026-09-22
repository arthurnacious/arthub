import { describe, expect, it } from 'vitest'
import { createAuthStore } from './auth-store'

function fakeStorage() {
  const data = new Map<string, string>()
  return {
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => {
      data.set(key, value)
    },
    removeItem: (key: string) => {
      data.delete(key)
    },
  }
}

describe('auth store', () => {
  it('starts with no token when nothing is stored', () => {
    const { store } = createAuthStore(fakeStorage())

    expect(store.state.token).toBeNull()
  })

  it('reads a previously stored token on creation', () => {
    const storage = fakeStorage()
    storage.setItem('arthub-auth-token', 'abc123')

    const { store } = createAuthStore(storage)

    expect(store.state.token).toBe('abc123')
  })

  it('login sets the token and persists it', () => {
    const storage = fakeStorage()
    const { store, login } = createAuthStore(storage)

    login('new-token')

    expect(store.state.token).toBe('new-token')
    expect(storage.getItem('arthub-auth-token')).toBe('new-token')
  })

  it('logout clears the token and removes it from storage', () => {
    const storage = fakeStorage()
    const { store, login, logout } = createAuthStore(storage)
    login('a-token')

    logout()

    expect(store.state.token).toBeNull()
    expect(storage.getItem('arthub-auth-token')).toBeNull()
  })

  it('works with no storage at all (SSR-safe)', () => {
    const { store, login } = createAuthStore(undefined)

    expect(store.state.token).toBeNull()
    expect(() => login('x')).not.toThrow()
    expect(store.state.token).toBe('x')
  })
})
