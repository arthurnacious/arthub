import { afterEach, describe, expect, it, vi } from 'vitest'
import { login } from './auth'
import * as http from './http'

describe('login', () => {
  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('posts credentials to /auth/login and returns the response', async () => {
    const apiPostSpy = vi
      .spyOn(http, 'apiPost')
      .mockResolvedValue({ token: 'a-token' })

    const result = await login('you@example.com', 'secret')

    expect(apiPostSpy).toHaveBeenCalledWith('/auth/login', {
      email: 'you@example.com',
      password: 'secret',
    })
    expect(result).toEqual({ token: 'a-token' })
  })
})
