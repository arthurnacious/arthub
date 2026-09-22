import { act, renderHook } from '@testing-library/react'
import { beforeEach, describe, expect, it } from 'vitest'
import { useSyncThemeColor } from './use-sync-theme-color'
import { themeStore } from '../stores/theme-store'

describe('useSyncThemeColor', () => {
  beforeEach(() => {
    themeStore.setColor('green')
    document.documentElement.removeAttribute('data-theme-color')
  })

  it('sets data-theme-color to the current store value on mount', () => {
    renderHook(() => useSyncThemeColor())

    expect(document.documentElement.dataset.themeColor).toBe('green')
  })

  it('updates the attribute when the store changes', () => {
    renderHook(() => useSyncThemeColor())

    act(() => {
      themeStore.setColor('purple')
    })

    expect(document.documentElement.dataset.themeColor).toBe('purple')
  })
})
