import { renderHook, waitFor } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { useAppReady } from './use-app-ready'

describe('useAppReady', () => {
  it('starts false, then becomes true after mount', async () => {
    const { result } = renderHook(() => useAppReady())

    await waitFor(() => expect(result.current).toBe(true))
  })
})
