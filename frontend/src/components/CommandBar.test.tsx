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
import { CommandBar } from './CommandBar'
import { ROUTES } from '../lib/routes'

function createTestRouter(initialPath: string) {
  const rootRoute = createRootRoute({ component: CommandBar })
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

// Router matching is async, so the ':' keydown listener isn't registered
// until CommandBar actually mounts. Wait for the always-rendered mount
// marker before exercising the shortcut, otherwise the keydown fires into
// a document with no listener attached yet.
async function openCommandBar() {
  await screen.findByTestId('command-bar-mounted')
  fireEvent.keyDown(document, { key: ':' })
  return screen.findByLabelText('Command')
}

describe('CommandBar', () => {
  it('is hidden until the ":" key is pressed', async () => {
    const router = createTestRouter('/')
    render(<RouterProvider router={router} />)
    await screen.findByTestId('command-bar-mounted')

    expect(screen.queryByLabelText('Command')).not.toBeInTheDocument()

    const input = await openCommandBar()

    expect(input).toHaveFocus()
  })

  it('ls lists the route names and stays open', async () => {
    const user = userEvent.setup()
    const router = createTestRouter('/')
    render(<RouterProvider router={router} />)

    const input = await openCommandBar()
    await user.type(input, 'ls{Enter}')

    for (const route of ROUTES) {
      expect(await screen.findByText(route.name)).toBeInTheDocument()
    }
    expect(screen.getByLabelText('Command')).toBeInTheDocument()
  })

  it('cd <route> navigates and closes the bar', async () => {
    const user = userEvent.setup()
    const router = createTestRouter('/')
    render(<RouterProvider router={router} />)

    const input = await openCommandBar()
    await user.type(input, 'cd clients{Enter}')

    await waitFor(() =>
      expect(router.state.location.pathname).toBe('/clients'),
    )
    expect(screen.queryByLabelText('Command')).not.toBeInTheDocument()
  })

  it('shows an error for an unknown command and stays open', async () => {
    const user = userEvent.setup()
    const router = createTestRouter('/')
    render(<RouterProvider router={router} />)

    const input = await openCommandBar()
    await user.type(input, 'nope{Enter}')

    expect(await screen.findByText('unknown command: nope')).toBeInTheDocument()
    expect(screen.getByLabelText('Command')).toBeInTheDocument()
  })

  it('Escape closes the bar', async () => {
    const router = createTestRouter('/')
    render(<RouterProvider router={router} />)

    const input = await openCommandBar()
    fireEvent.keyDown(input, { key: 'Escape' })

    await waitFor(() =>
      expect(screen.queryByLabelText('Command')).not.toBeInTheDocument(),
    )
  })
})
