import { Outlet, createFileRoute, redirect } from '@tanstack/react-router'
import { authStore } from '../stores/auth-store'

// Pathless layout: /quotes (table, quotes.index.tsx) and /quotes/$quoteId
// (detail, quotes.$quoteId.tsx) are unrelated pages that just share this
// URL prefix - same pattern as clients.tsx/projects.tsx.
export const Route = createFileRoute('/quotes')({
  beforeLoad: () => {
    if (typeof window === 'undefined') return
    if (authStore.store.state.token === null) {
      throw redirect({ to: '/login' })
    }
  },
  component: () => <Outlet />,
})
