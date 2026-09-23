import { afterEach, describe, expect, it, vi } from 'vitest'
import { changePassword, fetchProfile, login, updateProfile } from './auth'
import * as http from './http'

afterEach(() => {
  vi.restoreAllMocks()
})

describe('login', () => {
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

const PROFILE = { id: '1', name: 'Dana', email: 'dana@example.com', role: 'admin' as const }

describe('fetchProfile', () => {
  it('gets /auth/me and returns the profile', async () => {
    const apiGetSpy = vi.spyOn(http, 'apiGet').mockResolvedValue(PROFILE)

    expect(await fetchProfile()).toEqual(PROFILE)
    expect(apiGetSpy).toHaveBeenCalledWith('/auth/me')
  })
})

describe('updateProfile', () => {
  it('patches /auth/me with the given name/email', async () => {
    const apiPatchSpy = vi.spyOn(http, 'apiPatch').mockResolvedValue(PROFILE)

    const result = await updateProfile({ name: 'Dana', email: 'dana@example.com' })

    expect(apiPatchSpy).toHaveBeenCalledWith('/auth/me', { name: 'Dana', email: 'dana@example.com' })
    expect(result).toEqual(PROFILE)
  })
})

describe('changePassword', () => {
  it('posts current/new password to /auth/change-password', async () => {
    const apiPostSpy = vi.spyOn(http, 'apiPost').mockResolvedValue({ message: 'Password updated.' })

    const result = await changePassword({ currentPassword: 'old', newPassword: 'new-password' })

    expect(apiPostSpy).toHaveBeenCalledWith('/auth/change-password', {
      currentPassword: 'old',
      newPassword: 'new-password',
    })
    expect(result).toEqual({ message: 'Password updated.' })
  })
})
