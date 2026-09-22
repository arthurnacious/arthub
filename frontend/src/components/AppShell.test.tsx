import { fireEvent, render, screen, waitFor } from '@testing-library/react'
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
import { AppShell } from './AppShell'
import { ROUTES } from '../lib/routes'
import * as activityApi from '../lib/api/activity'

function TestApp() {
  return <AppShell>content</AppShell>
}

function createTestRouter(initialPath = '/') {
  const rootRoute = createRootRoute({ component: TestApp })
  const childRoutes = ROUTES.map((route) =>
    createRoute({
      getParentRoute: () => rootRoute,
      path: route.path,
      component: () => null,
    }),
  )

  return createRouter({
    routeTree: rootRoute.addChildren(childRoutes),
    history: createMemoryHistory({ initialEntries: [initialPath] }),
  })
}

function renderApp(initialPath = '/') {
  const router = createTestRouter(initialPath)
  const queryClient = new QueryClient()

  render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )

  return { router }
}

function getAside() {
  return screen.getByText('ART').closest('aside') as HTMLElement
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('AppShell mobile nav', () => {
  it('is closed (translated off-canvas) by default', async () => {
    vi.spyOn(activityApi, 'fetchActivity').mockResolvedValue([])
    renderApp()
    await screen.findByText('ART')

    expect(getAside().className).toContain('-translate-x-full')
  })

  it('opens when the menu button is clicked', async () => {
    vi.spyOn(activityApi, 'fetchActivity').mockResolvedValue([])
    const user = userEvent.setup()
    renderApp()
    await screen.findByText('ART')

    await user.click(screen.getByRole('button', { name: /toggle navigation menu/i }))

    expect(getAside().className).toContain('translate-x-0')
  })

  it('closes when a nav link is clicked', async () => {
    vi.spyOn(activityApi, 'fetchActivity').mockResolvedValue([])
    const user = userEvent.setup()
    renderApp()
    await screen.findByText('ART')

    await user.click(screen.getByRole('button', { name: /toggle navigation menu/i }))
    expect(getAside().className).toContain('translate-x-0')

    await user.click(screen.getByRole('link', { name: /clients/i }))

    await waitFor(() =>
      expect(getAside().className).toContain('-translate-x-full'),
    )
  })

  it('closes on Escape', async () => {
    vi.spyOn(activityApi, 'fetchActivity').mockResolvedValue([])
    const user = userEvent.setup()
    renderApp()
    await screen.findByText('ART')

    await user.click(screen.getByRole('button', { name: /toggle navigation menu/i }))
    expect(getAside().className).toContain('translate-x-0')

    fireEvent.keyDown(document, { key: 'Escape' })

    await waitFor(() =>
      expect(getAside().className).toContain('-translate-x-full'),
    )
  })
})
