import { Outlet, createFileRoute } from '@tanstack/react-router'

// Pathless layout: /clients (list, clients.index.tsx) and /clients/$clientId
// (detail, clients.$clientId.tsx) are unrelated pages that just share this
// URL prefix - this route exists only so file-based routing can nest them,
// and renders nothing of its own.
export const Route = createFileRoute('/clients')({
  component: () => <Outlet />,
})
