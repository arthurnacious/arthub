import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import {
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from '@tanstack/react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { TopBar } from './TopBar'
import * as activityApi from '../lib/api/activity'

function renderTopBar(props: { onMenuClick: () => void; mobileNavOpen: boolean }) {
  const rootRoute = createRootRoute({ component: () => <TopBar {...props} /> })
  const router = createRouter({
    routeTree: rootRoute,
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
  const queryClient = new QueryClient()

  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('TopBar', () => {
  it('calls onMenuClick when the menu button is clicked', async () => {
    vi.spyOn(activityApi, 'fetchActivity').mockResolvedValue([])
    const user = userEvent.setup()
    const onMenuClick = vi.fn()
    renderTopBar({ onMenuClick, mobileNavOpen: false })

    await user.click(
      await screen.findByRole('button', { name: /toggle navigation menu/i }),
    )

    expect(onMenuClick).toHaveBeenCalledOnce()
  })

  it('reflects the closed state on the menu button', async () => {
    vi.spyOn(activityApi, 'fetchActivity').mockResolvedValue([])
    renderTopBar({ onMenuClick: () => {}, mobileNavOpen: false })

    expect(
      await screen.findByRole('button', { name: /toggle navigation menu/i }),
    ).toHaveAttribute('aria-expanded', 'false')
  })

  it('reflects the open state on the menu button', async () => {
    vi.spyOn(activityApi, 'fetchActivity').mockResolvedValue([])
    renderTopBar({ onMenuClick: () => {}, mobileNavOpen: true })

    expect(
      await screen.findByRole('button', { name: /toggle navigation menu/i }),
    ).toHaveAttribute('aria-expanded', 'true')
  })
})
