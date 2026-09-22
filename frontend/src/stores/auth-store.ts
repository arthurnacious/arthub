import { Store } from '@tanstack/store'

export interface AuthState {
  token: string | null
}

const STORAGE_KEY = 'arthub-auth-token'

/**
 * Factory rather than a bare singleton so tests can create isolated
 * instances (their own Store + their own fake storage), same pattern as
 * theme-store.ts. The app itself only ever uses the single `authStore`
 * export below.
 */
export function createAuthStore(
  storage:
    | Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>
    | undefined = typeof window === 'undefined' ? undefined : window.localStorage,
) {
  const store = new Store<AuthState>({ token: storage?.getItem(STORAGE_KEY) ?? null })

  function login(token: string): void {
    store.setState(() => ({ token }))
    storage?.setItem(STORAGE_KEY, token)
  }

  function logout(): void {
    store.setState(() => ({ token: null }))
    storage?.removeItem(STORAGE_KEY)
  }

  return { store, login, logout }
}

export const authStore = createAuthStore()
