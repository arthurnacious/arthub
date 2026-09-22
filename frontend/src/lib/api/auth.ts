import { apiPost } from './http'

export interface LoginResponse {
  token: string
}

export function login(email: string, password: string): Promise<LoginResponse> {
  return apiPost<LoginResponse>('/auth/login', { email, password })
}
