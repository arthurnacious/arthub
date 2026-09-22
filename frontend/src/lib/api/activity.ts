import { apiGet } from './http'

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
// writes one of these; this is what backs the terminal-style kernel log
// panel and the notifications dropdown.
export async function fetchActivity(): Promise<ActivityEntry[]> {
  const { data } = await apiGet<{ data: ActivityEntry[] }>('/activity-log')
  return data
}
