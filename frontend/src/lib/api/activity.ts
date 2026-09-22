import { apiGet } from './http'
import type { PaginatedResult, PaginationParams } from './pagination'

export interface ActivityEntry {
  id: string
  user_id: string | null
  action: string
  description: string
  subject_type: string | null
  subject_id: string | null
  metadata: Record<string, unknown> | null
  created_at: string
}

// GET /activity-log - see backend/src/Modules/Activity/ActivityController.php.
// Every interesting backend action (client created/updated/deleted, etc.)
// writes one of these; this is what backs the notifications dropdown
// (unpaginated - just recent entries).
export async function fetchActivity(): Promise<ActivityEntry[]> {
  const { data } = await apiGet<{ data: ActivityEntry[] }>('/activity-log')
  return data
}

// Same endpoint, but paginated/searched server-side - backs the
// dashboard's audit log panel.
export function fetchActivityLog({
  page,
  pageSize,
  search,
}: PaginationParams): Promise<PaginatedResult<ActivityEntry>> {
  const params = new URLSearchParams({
    page: String(page),
    pageSize: String(pageSize),
    q: search,
  })

  return apiGet<PaginatedResult<ActivityEntry>>(`/activity-log?${params.toString()}`)
}
