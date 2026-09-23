import { afterEach, describe, expect, it, vi } from 'vitest'
import {
  createProject,
  deleteProject,
  deleteProjectImage,
  fetchProject,
  fetchProjectCategories,
  fetchProjects,
  updateProject,
  uploadProjectImages,
} from './projects'
import type { Project, ProjectDetail, ProjectInput } from './projects'
import { ApiError } from './http'
import * as http from './http'

const PROJECT: Project = {
  id: '1',
  clientId: 'c1',
  clientName: 'Nimbus Retail',
  name: 'Marketing Site',
  description: 'A site',
  category: 'web',
  url: 'https://example.com',
  status: 'active',
  coverImage: null,
}

const PROJECT_DETAIL: ProjectDetail = {
  id: '1',
  clientId: 'c1',
  clientName: 'Nimbus Retail',
  name: 'Marketing Site',
  description: 'A site',
  category: 'web',
  url: 'https://example.com',
  status: 'active',
  images: [],
}

const INPUT: ProjectInput = {
  clientId: 'c1',
  name: 'Marketing Site',
  description: 'A site',
  category: 'web',
  url: 'https://example.com',
  status: 'active',
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('fetchProjects', () => {
  it('requests the right query string and returns the result as-is', async () => {
    const paginated = { data: [PROJECT], page: 2, pageSize: 12, total: 30 }
    const apiGetSpy = vi.spyOn(http, 'apiGet').mockResolvedValue(paginated)

    const result = await fetchProjects({ page: 2, pageSize: 12, search: 'nimbus' })

    expect(apiGetSpy).toHaveBeenCalledWith('/projects?page=2&pageSize=12&q=nimbus')
    expect(result).toEqual(paginated)
  })

  it('includes category in the query string when given', async () => {
    const apiGetSpy = vi
      .spyOn(http, 'apiGet')
      .mockResolvedValue({ data: [], page: 1, pageSize: 12, total: 0 })

    await fetchProjects({ page: 1, pageSize: 12, search: '', category: 'branding' })

    expect(apiGetSpy).toHaveBeenCalledWith('/projects?page=1&pageSize=12&q=&category=branding')
  })

  it('omits category from the query string when empty', async () => {
    const apiGetSpy = vi
      .spyOn(http, 'apiGet')
      .mockResolvedValue({ data: [], page: 1, pageSize: 12, total: 0 })

    await fetchProjects({ page: 1, pageSize: 12, search: '' })

    expect(apiGetSpy).toHaveBeenCalledWith('/projects?page=1&pageSize=12&q=')
  })
})

describe('fetchProjectCategories', () => {
  it('unwraps the data array', async () => {
    vi.spyOn(http, 'apiGet').mockResolvedValue({ data: ['branding', 'web'] })

    expect(await fetchProjectCategories()).toEqual(['branding', 'web'])
  })
})

describe('fetchProject', () => {
  it('returns the project detail on success', async () => {
    vi.spyOn(http, 'apiGet').mockResolvedValue(PROJECT_DETAIL)

    expect(await fetchProject('1')).toEqual(PROJECT_DETAIL)
  })

  it('returns null (not a throw) on a 404', async () => {
    vi.spyOn(http, 'apiGet').mockRejectedValue(new ApiError('Project not found', 404))

    expect(await fetchProject('nonexistent')).toBeNull()
  })

  it('rethrows non-404 errors', async () => {
    vi.spyOn(http, 'apiGet').mockRejectedValue(new ApiError('Server error', 500))

    await expect(fetchProject('1')).rejects.toBeInstanceOf(ApiError)
  })
})

describe('createProject', () => {
  it('posts the input and returns the created project', async () => {
    const apiPostSpy = vi.spyOn(http, 'apiPost').mockResolvedValue(PROJECT_DETAIL)

    const result = await createProject(INPUT)

    expect(apiPostSpy).toHaveBeenCalledWith('/projects', INPUT)
    expect(result).toEqual(PROJECT_DETAIL)
  })
})

describe('updateProject', () => {
  it('patches the given id with the given fields', async () => {
    const apiPatchSpy = vi.spyOn(http, 'apiPatch').mockResolvedValue(PROJECT_DETAIL)

    const result = await updateProject('1', INPUT)

    expect(apiPatchSpy).toHaveBeenCalledWith('/projects/1', INPUT)
    expect(result).toEqual(PROJECT_DETAIL)
  })
})

describe('deleteProject', () => {
  it('sends a DELETE for the given id', async () => {
    const apiDeleteSpy = vi.spyOn(http, 'apiDelete').mockResolvedValue(undefined)

    await deleteProject('1')

    expect(apiDeleteSpy).toHaveBeenCalledWith('/projects/1')
  })
})

describe('uploadProjectImages', () => {
  it('posts every file under the "images" field name', async () => {
    const apiPostFormSpy = vi
      .spyOn(http, 'apiPostForm')
      .mockResolvedValue({ data: [{ id: 'img1', url: 'http://x/img1.png', sortOrder: 0 }] })
    const file = new File(['bytes'], 'cover.png', { type: 'image/png' })

    const result = await uploadProjectImages('1', [file])

    expect(apiPostFormSpy).toHaveBeenCalledTimes(1)
    const [path, formData] = apiPostFormSpy.mock.calls[0]
    expect(path).toBe('/projects/1/images')
    expect((formData as FormData).getAll('images')).toEqual([file])
    expect(result.data).toHaveLength(1)
  })
})

describe('deleteProjectImage', () => {
  it('sends a DELETE for the given project/image id pair', async () => {
    const apiDeleteSpy = vi.spyOn(http, 'apiDelete').mockResolvedValue(undefined)

    await deleteProjectImage('1', 'img1')

    expect(apiDeleteSpy).toHaveBeenCalledWith('/projects/1/images/img1')
  })
})
