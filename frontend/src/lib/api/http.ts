import { authStore } from '../../stores/auth-store'

const BASE_URL = import.meta.env.VITE_API_URL ?? 'http://localhost:9501'

export class ApiError extends Error {
  constructor(
    message: string,
    public readonly status: number,
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

interface ErrorBody {
  error?: string
  errors?: Record<string, string[]>
}

function errorMessage(body: ErrorBody | null, status: number): string {
  if (body?.error) return body.error
  if (body?.errors) return Object.values(body.errors).flat().join(', ')
  return `Request failed with status ${status}`
}

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const token = authStore.store.state.token
  // A FormData body needs the browser to set its own Content-Type (with
  // the multipart boundary) - setting it ourselves to application/json
  // would both be wrong and strip that boundary.
  const isFormData = init.body instanceof FormData

  const response = await fetch(`${BASE_URL}${path}`, {
    ...init,
    headers: {
      ...(isFormData ? {} : { 'Content-Type': 'application/json' }),
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...init.headers,
    },
  })

  if (!response.ok) {
    const body: ErrorBody | null = await response.json().catch(() => null)
    throw new ApiError(errorMessage(body, response.status), response.status)
  }

  if (response.status === 204) return undefined as T

  return response.json()
}

export function apiGet<T>(path: string): Promise<T> {
  return request<T>(path)
}

export function apiPost<T>(path: string, body: unknown): Promise<T> {
  return request<T>(path, { method: 'POST', body: JSON.stringify(body) })
}

export function apiPostForm<T>(path: string, formData: FormData): Promise<T> {
  return request<T>(path, { method: 'POST', body: formData })
}

export function apiPatch<T>(path: string, body: unknown): Promise<T> {
  return request<T>(path, { method: 'PATCH', body: JSON.stringify(body) })
}

export function apiDelete<T>(path: string): Promise<T> {
  return request<T>(path, { method: 'DELETE' })
}
