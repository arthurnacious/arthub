import { render, screen, waitFor } from '@testing-library/react'
import { fireEvent } from '@testing-library/react'
import {
  RouterProvider,
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
} from '@tanstack/react-router'
import { describe, expect, it } from 'vitest'
import { useVimNavigation } from './use-vim-navigation'
import { ROUTES } from '../lib/routes'

function Harness() {
  const { selectedIndex } = useVimNavigation()
  return (
    <div>
      <div data-testid="selected-index">{selectedIndex}</div>
      <input data-testid="text-input" />
    </div>
  )
}

function createTestRouter(initialPath: string) {
  const rootRoute = createRootRoute({ component: Harness })
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

describe('useVimNavigation', () => {
  it('starts selection on the currently active route', async () => {
    const router = createTestRouter('/projects')
    render(<RouterProvider router={router} />)

    await waitFor(() =>
      expect(screen.getByTestId('selected-index').textContent).toBe(
        String(ROUTES.findIndex((r) => r.path === '/projects')),
      ),
    )
  })

  it('j/k move the selection, clamped to the route list', async () => {
    const router = createTestRouter('/')
    render(<RouterProvider router={router} />)
    await screen.findByTestId('selected-index')

    fireEvent.keyDown(document, { key: 'j' })
    expect(screen.getByTestId('selected-index').textContent).toBe('1')

    fireEvent.keyDown(document, { key: 'k' })
    expect(screen.getByTestId('selected-index').textContent).toBe('0')

    fireEvent.keyDown(document, { key: 'k' })
    expect(screen.getByTestId('selected-index').textContent).toBe('0')
  })

  it('G then Enter navigates to the last route', async () => {
    const router = createTestRouter('/')
    render(<RouterProvider router={router} />)
    await screen.findByTestId('selected-index')

    fireEvent.keyDown(document, { key: 'G' })
    fireEvent.keyDown(document, { key: 'Enter' })

    await waitFor(() =>
      expect(router.state.location.pathname).toBe(
        ROUTES[ROUTES.length - 1].path,
      ),
    )
  })

  it('h goes back in history', async () => {
    const router = createTestRouter('/')
    render(<RouterProvider router={router} />)
    await screen.findByTestId('selected-index')

    await router.navigate({ to: '/clients' })
    await waitFor(() =>
      expect(router.state.location.pathname).toBe('/clients'),
    )

    fireEvent.keyDown(document, { key: 'h' })

    await waitFor(() => expect(router.state.location.pathname).toBe('/'))
  })

  it('ignores shortcuts while a text input is focused', async () => {
    const router = createTestRouter('/')
    render(<RouterProvider router={router} />)
    await screen.findByTestId('selected-index')

    screen.getByTestId('text-input').focus()
    fireEvent.keyDown(screen.getByTestId('text-input'), { key: 'j' })

    expect(screen.getByTestId('selected-index').textContent).toBe('0')
  })
})
