import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiError, apiDelete, apiGet, apiPatch, apiPost } from './http'
import { authStore } from '../../stores/auth-store'

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })
}

describe('http client', () => {
  beforeEach(() => {
    authStore.logout()
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('GETs and parses JSON', async () => {
    const fetchMock = vi.fn().mockImplementation(() => jsonResponse({ hello: 'world' }))
    vi.stubGlobal('fetch', fetchMock)

    const result = await apiGet<{ hello: string }>('/clients')

    expect(result).toEqual({ hello: 'world' })
    expect(fetchMock.mock.calls[0][0]).toContain('/clients')
  })

  it('attaches the auth token when logged in', async () => {
    authStore.login('a-token')
    const fetchMock = vi.fn().mockImplementation(() => jsonResponse({}))
    vi.stubGlobal('fetch', fetchMock)

    await apiGet('/clients')

    const [, init] = fetchMock.mock.calls[0]
    expect((init.headers as Record<string, string>).Authorization).toBe('Bearer a-token')
  })

  it('omits the Authorization header when logged out', async () => {
    const fetchMock = vi.fn().mockImplementation(() => jsonResponse({}))
    vi.stubGlobal('fetch', fetchMock)

    await apiGet('/clients')

    const [, init] = fetchMock.mock.calls[0]
    expect((init.headers as Record<string, string>).Authorization).toBeUndefined()
  })

  it('sends a JSON body on POST', async () => {
    const fetchMock = vi.fn().mockImplementation(() => jsonResponse({ id: '1' }, 201))
    vi.stubGlobal('fetch', fetchMock)

    await apiPost('/clients', { company: 'Nimbus' })

    const [, init] = fetchMock.mock.calls[0]
    expect(init.method).toBe('POST')
    expect(init.body).toBe(JSON.stringify({ company: 'Nimbus' }))
  })

  it('sends PATCH and DELETE with the right method', async () => {
    const fetchMock = vi.fn().mockImplementation(() => jsonResponse({}))
    vi.stubGlobal('fetch', fetchMock)

    await apiPatch('/clients/1', { company: 'X' })
    await apiDelete('/clients/1')

    expect(fetchMock.mock.calls[0][1].method).toBe('PATCH')
    expect(fetchMock.mock.calls[1][1].method).toBe('DELETE')
  })

  it('throws an ApiError with the backend error message on a non-OK response', async () => {
    const fetchMock = vi.fn().mockImplementation(() => jsonResponse({ error: 'Client not found' }, 404))
    vi.stubGlobal('fetch', fetchMock)

    await expect(apiGet('/clients/nope')).rejects.toMatchObject({
      message: 'Client not found',
      status: 404,
    })
  })

  it('flattens DTO validation errors into the ApiError message', async () => {
    const fetchMock = vi
      .fn()
      .mockImplementation(() => jsonResponse({ errors: { email: ['Must be a valid email'] } }, 422))
    vi.stubGlobal('fetch', fetchMock)

    await expect(apiPost('/clients', {})).rejects.toMatchObject({
      message: expect.stringContaining('Must be a valid email'),
    })
    await expect(apiPost('/clients', {})).rejects.toBeInstanceOf(ApiError)
  })
})
