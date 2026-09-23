import { Outlet, createFileRoute, redirect } from '@tanstack/react-router'
import { authStore } from '../stores/auth-store'

// Pathless layout: /invoices (table, invoices.index.tsx) and
// /invoices/$invoiceId (detail, invoices.$invoiceId.tsx) are unrelated
// pages that just share this URL prefix - same pattern as
// clients.tsx/projects.tsx/quotes.tsx.
export const Route = createFileRoute('/invoices')({
  beforeLoad: () => {
    if (typeof window === 'undefined') return
    if (authStore.store.state.token === null) {
      throw redirect({ to: '/login' })
    }
  },
  component: () => <Outlet />,
})
