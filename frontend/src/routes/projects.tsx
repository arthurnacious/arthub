import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/projects')({ component: ProjectsPage })

function ProjectsPage() {
  return (
    <div>
      <h1 className="glow-text text-lg font-bold text-[var(--accent)]">PROJECTS</h1>
      <p className="mt-2 text-sm text-[var(--text-dim)]">
        // coming soon: project list wired to the backend
      </p>
    </div>
  )
}
