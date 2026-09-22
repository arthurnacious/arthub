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
import { NotificationsMenu } from './NotificationsMenu'
import * as activityApi from '../lib/api/activity'
import type { ActivityEntry } from '../lib/api/activity'

const CLIENT_EVENT: ActivityEntry = {
  id: '1',
  user_id: 'u1',
  action: 'client.created',
  description: 'Created client Nimbus Retail',
  subject_type: 'client',
  subject_id: 'c1',
  metadata: null,
  created_at: new Date().toISOString(),
}

const LOGIN_EVENT: ActivityEntry = {
  id: '2',
  user_id: 'u1',
  action: 'auth.login',
  description: 'Admin logged in',
  subject_type: null,
  subject_id: null,
  metadata: null,
  created_at: new Date().toISOString(),
}

function renderWithProviders(initialPath = '/') {
  const rootRoute = createRootRoute({ component: NotificationsMenu })
  const clientRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/clients/$clientId',
    component: () => null,
  })

  const router = createRouter({
    routeTree: rootRoute.addChildren([clientRoute]),
    history: createMemoryHistory({ initialEntries: [initialPath] }),
  })
  const queryClient = new QueryClient()

  render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )

  return { router }
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('NotificationsMenu', () => {
  it('shows the count of real activity entries on the toggle button', async () => {
    vi.spyOn(activityApi, 'fetchActivity').mockResolvedValue([CLIENT_EVENT, LOGIN_EVENT])
    renderWithProviders()

    expect(
      await screen.findByRole('button', { name: /notifications \(2\)/i }),
    ).toBeInTheDocument()
  })

  it('is closed until clicked', async () => {
    vi.spyOn(activityApi, 'fetchActivity').mockResolvedValue([CLIENT_EVENT])
    renderWithProviders()

    expect(screen.queryByText('notifications')).not.toBeInTheDocument()
  })

  it('opens the panel and lists real activity descriptions', async () => {
    const user = userEvent.setup()
    vi.spyOn(activityApi, 'fetchActivity').mockResolvedValue([CLIENT_EVENT])
    renderWithProviders()

    await user.click(await screen.findByRole('button', { name: /notifications/i }))

    expect(screen.getByText('Created client Nimbus Retail')).toBeInTheDocument()
  })

  it('navigates to the client on click for a client-subject entry', async () => {
    const user = userEvent.setup()
    vi.spyOn(activityApi, 'fetchActivity').mockResolvedValue([CLIENT_EVENT])
    const { router } = renderWithProviders()

    await user.click(await screen.findByRole('button', { name: /notifications/i }))
    await user.click(screen.getByText('Created client Nimbus Retail'))

    await waitFor(() => expect(router.state.location.pathname).toBe('/clients/c1'))
    expect(screen.queryByText('notifications')).not.toBeInTheDocument()
  })

  it('renders a non-navigable entry as plain text, not a button', async () => {
    const user = userEvent.setup()
    vi.spyOn(activityApi, 'fetchActivity').mockResolvedValue([LOGIN_EVENT])
    renderWithProviders()

    await user.click(await screen.findByRole('button', { name: /notifications/i }))

    expect(
      screen.queryByRole('button', { name: /admin logged in/i }),
    ).not.toBeInTheDocument()
    expect(screen.getByText('Admin logged in')).toBeInTheDocument()
  })

  it('closes on Escape', async () => {
    const user = userEvent.setup()
    vi.spyOn(activityApi, 'fetchActivity').mockResolvedValue([CLIENT_EVENT])
    renderWithProviders()

    await user.click(await screen.findByRole('button', { name: /notifications/i }))
    expect(screen.getByText('notifications')).toBeInTheDocument()

    fireEvent.keyDown(document, { key: 'Escape' })

    expect(screen.queryByText('notifications')).not.toBeInTheDocument()
  })

  it('shows a placeholder with no activity yet', async () => {
    const user = userEvent.setup()
    vi.spyOn(activityApi, 'fetchActivity').mockResolvedValue([])
    renderWithProviders()

    await user.click(await screen.findByRole('button', { name: /notifications \(0\)/i }))

    expect(screen.getByText('-- no notifications --')).toBeInTheDocument()
  })
})
