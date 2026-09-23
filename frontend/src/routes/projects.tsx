import { Outlet, createFileRoute, redirect } from '@tanstack/react-router'
import { authStore } from '../stores/auth-store'

// Pathless layout: /projects (grid, projects.index.tsx) and
// /projects/$projectId (detail, projects.$projectId.tsx) are unrelated
// pages that just share this URL prefix - this route exists only so
// file-based routing can nest them, plus the auth guard below (it covers
// both children, since it's their shared parent). Same pattern as
// clients.tsx.
export const Route = createFileRoute('/projects')({
  beforeLoad: () => {
    if (typeof window === 'undefined') return
    if (authStore.store.state.token === null) {
      throw redirect({ to: '/login' })
    }
  },
  component: () => <Outlet />,
})
