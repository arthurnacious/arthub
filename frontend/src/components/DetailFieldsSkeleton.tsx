import { Skeleton } from './Skeleton'

// A generic "label: value" field list placeholder - used by every detail
// page (Client/Project/Quote/Invoice) while its record is still loading.
export function DetailFieldsSkeleton({ rows = 4 }: { rows?: number }) {
  return (
    <div className="flex flex-col gap-3">
      {Array.from({ length: rows }).map((_, i) => (
        <div key={i} className="flex items-center gap-2">
          <Skeleton className="h-3 w-16" />
          <Skeleton className="h-3 w-32" />
        </div>
      ))}
    </div>
  )
}
