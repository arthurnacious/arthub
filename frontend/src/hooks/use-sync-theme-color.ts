import { useEffect } from 'react'
import { useStore } from '@tanstack/react-store'
import { themeStore } from '../stores/theme-store'

/**
 * Keeps [data-theme-color] on <html> in sync with the theme store, so the
 * CSS custom properties in styles.css (--accent / --accent-dim) pick up
 * the right color. Call this once, at the app root.
 */
export function useSyncThemeColor(): void {
  const color = useStore(themeStore.store, (state) => state)

  useEffect(() => {
    document.documentElement.dataset.themeColor = color
  }, [color])
}
