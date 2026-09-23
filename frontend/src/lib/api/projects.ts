import { apiGet } from './http'
import type { PaginatedResult, PaginationParams } from './pagination'

// Matches ProjectsController::present() - see
// backend/src/Modules/Projects/ProjectsController.php. `url` is only set
// for a published, linkable site; its absence means "just show the image
// gallery" (e.g. logo work).
export interface Project {
  id: string
  clientId: string
  clientName: string
  name: string
  description: string
  category: string
  url: string | null
  status: 'active' | 'completed' | 'archived'
  coverImage: string | null
}

// GET /projects?page=&pageSize=&q=&category= - server-side pagination,
// search and category filter.
export function fetchProjects({
  page,
  pageSize,
  search,
  category = '',
}: PaginationParams & { category?: string }): Promise<PaginatedResult<Project>> {
  const params = new URLSearchParams({
    page: String(page),
    pageSize: String(pageSize),
    q: search,
    ...(category ? { category } : {}),
  })

  return apiGet<PaginatedResult<Project>>(`/projects?${params.toString()}`)
}

export function fetchProjectCategories(): Promise<string[]> {
  return apiGet<{ data: string[] }>('/projects/categories').then((body) => body.data)
}
