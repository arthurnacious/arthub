import { Store } from '@tanstack/store'

// The terminal-log accent color, toggled by the user. Green is the
// default; cycling goes green -> purple -> orange -> red -> green.
export type ThemeColor = 'green' | 'purple' | 'orange' | 'red'

export const THEME_COLORS: ThemeColor[] = ['green', 'purple', 'orange', 'red']

const STORAGE_KEY = 'arthub-theme-color'

function isThemeColor(value: unknown): value is ThemeColor {
  return typeof value === 'string' && (THEME_COLORS as string[]).includes(value)
}

function readStoredColor(storage: Pick<Storage, 'getItem'> | undefined): ThemeColor {
  const stored = storage?.getItem(STORAGE_KEY) ?? null
  return isThemeColor(stored) ? stored : 'green'
}

/**
 * Factory rather than a bare singleton so tests can create isolated
 * instances (their own Store + their own fake storage) instead of fighting
 * over module-level state and real localStorage between test runs. The app
 * itself only ever uses the single `themeStore` export below.
 */
export function createThemeStore(
  storage: Pick<Storage, 'getItem' | 'setItem'> | undefined = typeof window === 'undefined'
    ? undefined
    : window.localStorage,
) {
  const store = new Store<ThemeColor>(readStoredColor(storage))

  function setColor(color: ThemeColor): void {
    store.setState(() => color)
    storage?.setItem(STORAGE_KEY, color)
  }

  function cycleColor(): void {
    const currentIndex = THEME_COLORS.indexOf(store.state)
    const nextIndex = (currentIndex + 1) % THEME_COLORS.length
    setColor(THEME_COLORS[nextIndex])
  }

  return { store, setColor, cycleColor }
}

export const themeStore = createThemeStore()
