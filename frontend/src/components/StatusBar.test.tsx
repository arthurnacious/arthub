import { act, render, screen } from '@testing-library/react'
import {
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRouter,
} from '@tanstack/react-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { StatusBar } from './StatusBar'
import { themeStore } from '../stores/theme-store'

function createTestRouter(initialPath: string) {
  const rootRoute = createRootRoute({ component: StatusBar })

  return createRouter({
    routeTree: rootRoute,
    history: createMemoryHistory({ initialEntries: [initialPath] }),
  })
}

describe('StatusBar', () => {
  beforeEach(() => {
    themeStore.setColor('purple')
  })

  it('shows a live clock that starts ticking after mount', async () => {
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-06-01T10:40:40Z'))
    const router = createTestRouter('/')
    render(<RouterProvider router={router} />)

    await act(async () => {
      await vi.advanceTimersByTimeAsync(0)
    })

    expect(screen.getByText(/^\d{2}:\d{2}:\d{2}$/)).toBeInTheDocument()

    const before = screen.getByText(/^\d{2}:\d{2}:\d{2}$/).textContent

    await act(async () => {
      await vi.advanceTimersByTimeAsync(2000)
    })

    const after = screen.getByText(/^\d{2}:\d{2}:\d{2}$/).textContent
    expect(after).not.toBe(before)

    vi.useRealTimers()
  })

  it('shows the current route', async () => {
    const router = createTestRouter('/clients')
    render(<RouterProvider router={router} />)

    expect(await screen.findByText('/clients')).toBeInTheDocument()
  })

  it('shows the active theme scheme', async () => {
    const router = createTestRouter('/')
    render(<RouterProvider router={router} />)

    expect(await screen.findByText('purple')).toBeInTheDocument()
  })
})
