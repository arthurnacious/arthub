import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/invoices')({ component: InvoicesPage })

function InvoicesPage() {
  return (
    <div>
      <h1 className="glow-text text-lg font-bold text-[var(--accent)]">INVOICES</h1>
      <p className="mt-2 text-sm text-[var(--text-dim)]">
        // coming soon: invoice list wired to the backend
      </p>
    </div>
  )
}
