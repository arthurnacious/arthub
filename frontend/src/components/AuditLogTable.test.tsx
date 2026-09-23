import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import {
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from '@tanstack/react-router'
import { describe, expect, it } from 'vitest'
import { AuditLogTable } from './AuditLogTable'
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

function renderWithProviders(entries: ActivityEntry[], isLoading = false) {
  const rootRoute = createRootRoute({
    component: () => <AuditLogTable entries={entries} isLoading={isLoading} />,
  })
  const clientRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/clients/$clientId',
    component: () => null,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([clientRoute]),
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })

  render(<RouterProvider router={router} />)

  return { router }
}

describe('AuditLogTable', () => {
  it('renders a row per entry with its action and description', async () => {
    renderWithProviders([CLIENT_EVENT])

    expect(await screen.findByText('client.created')).toBeInTheDocument()
    expect(screen.getByText('Created client Nimbus Retail')).toBeInTheDocument()
  })

  it('shows a placeholder when there are no entries', async () => {
    renderWithProviders([])

    expect(await screen.findByText('-- no activity yet --')).toBeInTheDocument()
  })

  it('shows skeleton rows instead of the empty placeholder while loading', async () => {
    renderWithProviders([], true)

    expect(await screen.findAllByRole('row')).toHaveLength(6) // 1 header + 5 skeleton rows
    expect(screen.queryByText('-- no activity yet --')).not.toBeInTheDocument()
  })

  it('navigates when a row with a known subject is clicked', async () => {
    const user = userEvent.setup()
    const { router } = renderWithProviders([CLIENT_EVENT])

    await user.click(await screen.findByText('Created client Nimbus Retail'))

    expect(router.state.location.pathname).toBe('/clients/c1')
  })

  it('is not clickable for an entry with no known subject', async () => {
    renderWithProviders([LOGIN_EVENT])

    expect(
      screen.queryByRole('button', { name: /admin logged in/i }),
    ).not.toBeInTheDocument()
    expect(await screen.findByText('Admin logged in')).toBeInTheDocument()
  })
})
