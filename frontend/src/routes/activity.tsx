import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/activity')({ component: ActivityPage })

function ActivityPage() {
  return (
    <div>
      <h1 className="glow-text text-lg font-bold text-[var(--accent)]">ACTIVITY</h1>
      <p className="mt-2 text-sm text-[var(--text-dim)]">
        // coming soon: live activity log wired to the backend
      </p>
    </div>
  )
}
