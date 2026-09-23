import { ApiError, apiDelete, apiGet, apiPatch, apiPost, apiPostForm } from './http'
import type { PaginatedResult, PaginationParams } from './pagination'
import type { StatusTone } from '../../components/StatusBadge'

export type ProjectStatus = 'active' | 'completed' | 'archived'

export const PROJECT_STATUS_TONE: Record<ProjectStatus, StatusTone> = {
  active: 'success',
  completed: 'info',
  archived: 'neutral',
}

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
  status: ProjectStatus
  coverImage: string | null
}

export interface ProjectImage {
  id: string
  url: string
  sortOrder: number
}

// Matches ProjectsController::presentDetail() - the same fields as
// Project, but with every image (not just a cover) instead of coverImage.
export interface ProjectDetail {
  id: string
  clientId: string
  clientName: string
  name: string
  description: string
  category: string
  url: string | null
  status: ProjectStatus
  images: ProjectImage[]
}

export interface ProjectInput {
  clientId: string
  name: string
  description: string
  category: string
  url: string | null
  status: ProjectStatus
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

export async function fetchProject(id: string): Promise<ProjectDetail | null> {
  try {
    return await apiGet<ProjectDetail>(`/projects/${id}`)
  } catch (error) {
    if (error instanceof ApiError && error.status === 404) return null
    throw error
  }
}

export function createProject(input: ProjectInput): Promise<ProjectDetail> {
  return apiPost<ProjectDetail>('/projects', input)
}

export function updateProject(id: string, input: ProjectInput): Promise<ProjectDetail> {
  return apiPatch<ProjectDetail>(`/projects/${id}`, input)
}

export async function deleteProject(id: string): Promise<void> {
  await apiDelete(`/projects/${id}`)
}

// POST /projects/{id}/images (multipart) - see
// ProjectsController::uploadImages(). Field name must be "images" to
// match `$request->file('images')` on the backend.
export function uploadProjectImages(id: string, files: File[]): Promise<{ data: ProjectImage[] }> {
  const formData = new FormData()
  for (const file of files) {
    formData.append('images', file)
  }

  return apiPostForm<{ data: ProjectImage[] }>(`/projects/${id}/images`, formData)
}

export async function deleteProjectImage(projectId: string, imageId: string): Promise<void> {
  await apiDelete(`/projects/${projectId}/images/${imageId}`)
}
