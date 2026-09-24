import { AsciiFallback } from './AsciiFallback'
import { StatusBadge } from './StatusBadge'
import { PROJECT_STATUS_TONE } from '../lib/api/projects'
import type { Project } from '../lib/api/projects'

// One project, many images - this shows only the cover (the first
// uploaded image, or a generated ASCII fingerprint if there isn't one
// yet). Clicking through to the detail page is where the rest of the
// gallery, the slider, and the published-site link (if any) live.
export function ProjectCard({ project, onClick }: { project: Project; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="glow-border group flex flex-col overflow-hidden rounded border border-[var(--border)] bg-[var(--panel)] text-left transition-colors hover:border-[var(--accent)]"
    >
      <div className="flex aspect-video items-center justify-center overflow-hidden border-b border-[var(--border)] bg-black/30 px-2">
        {project.coverImage ? (
          <img
            src={project.coverImage}
            alt={project.name}
            className="h-full w-full object-cover transition-transform duration-200 group-hover:scale-105"
          />
        ) : (
          <AsciiFallback seed={project.id} className="text-[9px]" />
        )}
      </div>
      <div className="flex flex-col gap-2 p-3">
        <div className="flex items-center justify-between gap-2">
          <span className="min-w-0 truncate text-sm font-bold text-[var(--text)]">{project.name}</span>
          <StatusBadge label={project.status} tone={PROJECT_STATUS_TONE[project.status]} />
        </div>
        <div className="flex items-center justify-between gap-2 text-xs text-[var(--text-dim)]">
          <span className="min-w-0 truncate">{project.clientName}</span>
          <span className="shrink-0 rounded border border-[var(--border)] px-1.5 py-0.5 uppercase tracking-wider">
            {project.category}
          </span>
        </div>
      </div>
    </button>
  )
}
