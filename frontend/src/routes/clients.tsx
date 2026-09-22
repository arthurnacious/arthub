import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/clients')({ component: ClientsPage })

function ClientsPage() {
  return (
    <div>
      <h1 className="glow-text text-lg font-bold text-[var(--accent)]">CLIENTS</h1>
      <p className="mt-2 text-sm text-[var(--text-dim)]">
        // coming soon: client list wired to the backend
      </p>
    </div>
  )
}
