import { Skeleton } from './Skeleton'

// Matches ProjectCard's shape (cover area + name/status row + client/
// category row) so the grid doesn't visibly reflow once real cards
// replace these.
export function ProjectCardSkeleton() {
  return (
    <div className="flex flex-col overflow-hidden rounded border border-[var(--border)] bg-[var(--panel)]">
      <Skeleton className="aspect-video w-full rounded-none border-b border-[var(--border)]" />
      <div className="flex flex-col gap-2 p-3">
        <div className="flex items-center justify-between gap-2">
          <Skeleton className="h-4 w-2/3" />
          <Skeleton className="h-4 w-12" />
        </div>
        <div className="flex items-center justify-between gap-2">
          <Skeleton className="h-3 w-1/2" />
          <Skeleton className="h-3 w-14" />
        </div>
      </div>
    </div>
  )
}
