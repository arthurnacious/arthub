import { Outlet, createFileRoute, redirect } from '@tanstack/react-router'
import { authStore } from '../stores/auth-store'

// Pathless layout: /clients (list, clients.index.tsx) and /clients/$clientId
// (detail, clients.$clientId.tsx) are unrelated pages that just share this
// URL prefix - this route exists only so file-based routing can nest them,
// plus the auth guard below (it covers both children, since it's their
// shared parent).
//
// The guard only checks on the client: the token lives in localStorage,
// which the server can't see, so a server-side check would see "no token"
// on every request - even from a genuinely logged-in user - and always
// redirect. The backend enforces the real boundary regardless (every
// /clients endpoint requires a valid token); this is just a UX nicety on
// top of that.
export const Route = createFileRoute('/clients')({
  beforeLoad: () => {
    if (typeof window === 'undefined') return
    if (authStore.store.state.token === null) {
      throw redirect({ to: '/login' })
    }
  },
  component: () => <Outlet />,
})
