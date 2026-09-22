export interface CurrentUser {
  name: string
  email: string
  role: string
}

// Placeholder until the frontend has a real session/auth flow wired to the
// backend's AuthController (login/forgot-password/reset-password already
// exist there - see backend/src/Modules/Auth). Swap this out for the real
// authenticated user once that's built.
export const MOCK_CURRENT_USER: CurrentUser = {
  name: 'arthurnacious',
  email: 'freelancer@arthub.local',
  role: 'admin',
}
