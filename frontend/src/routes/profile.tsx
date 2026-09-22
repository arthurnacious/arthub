import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/profile')({ component: ProfilePage })

function ProfilePage() {
  return (
    <div>
      <h1 className="glow-text text-lg font-bold text-[var(--accent)]">PROFILE</h1>
      <p className="mt-2 text-sm text-[var(--text-dim)]">
        // coming soon: editable profile wired to the backend
      </p>
    </div>
  )
}
