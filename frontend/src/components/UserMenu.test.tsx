import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import {
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from '@tanstack/react-router'
import { describe, expect, it } from 'vitest'
import { UserMenu } from './UserMenu'
import { MOCK_CURRENT_USER } from '../lib/current-user'

function createTestRouter() {
  const rootRoute = createRootRoute({ component: UserMenu })
  const routes = ['/', '/profile', '/invoices'].map((path) =>
    createRoute({ getParentRoute: () => rootRoute, path, component: () => null }),
  )

  return createRouter({
    routeTree: rootRoute.addChildren(routes),
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
}

describe('UserMenu', () => {
  it('is closed until clicked', async () => {
    const router = createTestRouter()
    render(<RouterProvider router={router} />)

    expect(screen.queryByText(MOCK_CURRENT_USER.email)).not.toBeInTheDocument()
  })

  it('shows name, email and role when opened', async () => {
    const user = userEvent.setup()
    const router = createTestRouter()
    render(<RouterProvider router={router} />)

    await user.click(await screen.findByRole('button', { name: MOCK_CURRENT_USER.name }))

    expect(screen.getByText(MOCK_CURRENT_USER.email)).toBeInTheDocument()
    expect(screen.getByText(MOCK_CURRENT_USER.role)).toBeInTheDocument()
  })

  it('profile link navigates to /profile and closes the menu', async () => {
    const user = userEvent.setup()
    const router = createTestRouter()
    render(<RouterProvider router={router} />)

    await user.click(await screen.findByRole('button', { name: MOCK_CURRENT_USER.name }))
    await user.click(screen.getByRole('link', { name: /profile/i }))

    await waitFor(() => expect(router.state.location.pathname).toBe('/profile'))
    expect(screen.queryByText(MOCK_CURRENT_USER.email)).not.toBeInTheDocument()
  })

  it('my invoices link navigates to /invoices', async () => {
    const user = userEvent.setup()
    const router = createTestRouter()
    render(<RouterProvider router={router} />)

    await user.click(await screen.findByRole('button', { name: MOCK_CURRENT_USER.name }))
    await user.click(screen.getByRole('link', { name: /my invoices/i }))

    await waitFor(() => expect(router.state.location.pathname).toBe('/invoices'))
  })

  it('logout closes the menu', async () => {
    const user = userEvent.setup()
    const router = createTestRouter()
    render(<RouterProvider router={router} />)

    await user.click(await screen.findByRole('button', { name: MOCK_CURRENT_USER.name }))
    await user.click(screen.getByRole('button', { name: /logout/i }))

    expect(screen.queryByText(MOCK_CURRENT_USER.email)).not.toBeInTheDocument()
  })

  it('closes on Escape', async () => {
    const user = userEvent.setup()
    const router = createTestRouter()
    render(<RouterProvider router={router} />)

    await user.click(await screen.findByRole('button', { name: MOCK_CURRENT_USER.name }))
    fireEvent.keyDown(document, { key: 'Escape' })

    expect(screen.queryByText(MOCK_CURRENT_USER.email)).not.toBeInTheDocument()
  })
})
