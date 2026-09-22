import { useState } from 'react'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { Panel } from '../components/Panel'
import { Logo } from '../components/Logo'
import { login } from '../lib/api/auth'
import { ApiError } from '../lib/api/http'
import { authStore } from '../stores/auth-store'

export const Route = createFileRoute('/login')({ component: LoginPage })

const FIELD_CLASSES =
  'rounded border border-[var(--border)] bg-transparent px-2 py-1 text-sm text-[var(--text)] outline-none focus:border-[var(--accent)]'

function LoginPage() {
  const navigate = useNavigate()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  async function handleSubmit(event: React.FormEvent) {
    event.preventDefault()
    setError(null)
    setSubmitting(true)

    try {
      const { token } = await login(email, password)
      authStore.login(token)
      navigate({ to: '/' })
    } catch (err) {
      setError(err instanceof ApiError ? err.message : 'Something went wrong. Try again.')
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-[var(--bg)] p-4">
      <div className="w-full max-w-sm">
        <div className="mb-6 flex justify-center">
          <Logo className="text-xl" />
        </div>
        <Panel title="login">
          <form onSubmit={handleSubmit} className="flex flex-col gap-3">
            <label htmlFor="login-email" className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
              email
              <input
                id="login-email"
                type="email"
                value={email}
                onChange={(event) => setEmail(event.target.value)}
                required
                className={FIELD_CLASSES}
              />
            </label>
            <label htmlFor="login-password" className="flex flex-col gap-1 text-xs text-[var(--text-dim)]">
              password
              <input
                id="login-password"
                type="password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                required
                className={FIELD_CLASSES}
              />
            </label>
            {error && <p className="text-xs text-red-400">{error}</p>}
            <button
              type="submit"
              disabled={submitting}
              className="mt-2 rounded border border-[var(--accent)] bg-[var(--accent-dim)] px-3 py-1.5 text-xs text-[var(--accent)] transition-colors disabled:opacity-50"
            >
              {submitting ? 'logging in...' : 'log in'}
            </button>
          </form>
        </Panel>
      </div>
    </div>
  )
}
