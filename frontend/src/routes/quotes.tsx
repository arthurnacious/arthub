import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/quotes')({ component: QuotesPage })

function QuotesPage() {
  return (
    <div>
      <h1 className="glow-text text-lg font-bold text-[var(--accent)]">QUOTES</h1>
      <p className="mt-2 text-sm text-[var(--text-dim)]">
        // coming soon: quote list wired to the backend
      </p>
    </div>
  )
}
