import { apiGet, apiPatch, apiPost } from './http'

export interface LoginResponse {
  token: string
}

export function login(email: string, password: string): Promise<LoginResponse> {
  return apiPost<LoginResponse>('/auth/login', { email, password })
}

// Matches AuthController::present() - see
// backend/src/Modules/Auth/AuthController.php.
export interface Profile {
  id: string
  name: string
  email: string
  role: 'admin' | 'client'
}

export function fetchProfile(): Promise<Profile> {
  return apiGet<Profile>('/auth/me')
}

export function updateProfile(input: { name: string; email: string }): Promise<Profile> {
  return apiPatch<Profile>('/auth/me', input)
}

export function changePassword(input: {
  currentPassword: string
  newPassword: string
}): Promise<{ message: string }> {
  return apiPost('/auth/change-password', input)
}
