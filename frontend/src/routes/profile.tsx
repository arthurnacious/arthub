import { useEffect, useState } from 'react'
import { createFileRoute, redirect } from '@tanstack/react-router'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Panel } from '../components/Panel'
import { ProfileForm } from '../components/ProfileForm'
import { ChangePasswordForm } from '../components/ChangePasswordForm'
import { DetailFieldsSkeleton } from '../components/DetailFieldsSkeleton'
import { ErrorState } from '../components/ErrorState'
import { Skeleton } from '../components/Skeleton'
import { changePassword, fetchProfile, updateProfile } from '../lib/api/auth'
import { ApiError } from '../lib/api/http'
import { authStore } from '../stores/auth-store'

// Same client-only auth guard as clients.tsx/projects.tsx/etc - see those
// for why it's client-only (the token lives in localStorage).
export const Route = createFileRoute('/profile')({
  beforeLoad: () => {
    if (typeof window === 'undefined') return
    if (authStore.store.state.token === null) {
      throw redirect({ to: '/login' })
    }
  },
  component: ProfilePage,
})

function ProfilePage() {
  const queryClient = useQueryClient()

  const { data: profile, isPending, isError, refetch } = useQuery({
    queryKey: ['profile'],
    queryFn: fetchProfile,
  })

  const [form, setForm] = useState<{ name: string; email: string } | null>(null)

  useEffect(() => {
    if (profile) setForm({ name: profile.name, email: profile.email })
  }, [profile])

  const updateMutation = useMutation({
    mutationFn: (values: { name: string; email: string }) => updateProfile(values),
    onSuccess: (updated) => {
      queryClient.setQueryData(['profile'], updated)
    },
  })

  const [passwordFormKey, setPasswordFormKey] = useState(0)
  const [passwordMessage, setPasswordMessage] = useState<{ tone: 'success' | 'error'; text: string } | null>(
    null,
  )

  const passwordMutation = useMutation({
    mutationFn: (values: { currentPassword: string; newPassword: string }) => changePassword(values),
    onSuccess: () => {
      setPasswordMessage({ tone: 'success', text: 'Password updated.' })
      setPasswordFormKey((key) => key + 1)
    },
    onError: (error: unknown) => {
      setPasswordMessage({
        tone: 'error',
        text: error instanceof ApiError ? error.message : 'Something went wrong.',
      })
    },
  })

  if (isPending) {
    return (
      <div>
        <Skeleton className="h-6 w-32" />
        <div className="mt-6 max-w-md">
          <Panel title="account">
            <DetailFieldsSkeleton rows={2} />
          </Panel>
        </div>
      </div>
    )
  }

  if (isError) {
    return (
      <div>
        <h1 className="glow-text text-lg font-bold text-[var(--accent)]">PROFILE</h1>
        <div className="mt-6">
          <ErrorState
            title="COULDN'T LOAD PROFILE"
            message="Something went wrong fetching your profile."
            onRetry={() => refetch()}
          />
        </div>
      </div>
    )
  }

  if (!profile || !form) {
    return null
  }

  return (
    <div>
      <h1 className="glow-text text-lg font-bold text-[var(--accent)]">PROFILE</h1>
      <p className="mt-2 text-sm text-[var(--text-dim)]">// {profile.role}</p>

      <div className="mt-6 flex max-w-md flex-col gap-6">
        <Panel title="account">
          <ProfileForm
            values={form}
            onChange={setForm}
            onSubmit={() => updateMutation.mutate(form)}
            submitting={updateMutation.isPending}
          />
        </Panel>

        <Panel title="change password">
          <ChangePasswordForm
            key={passwordFormKey}
            onSubmit={(values) => passwordMutation.mutate(values)}
            submitting={passwordMutation.isPending}
          />
          {passwordMessage && (
            <p
              className={`mt-2 text-xs ${
                passwordMessage.tone === 'error' ? 'text-red-400' : 'text-[var(--accent)]'
              }`}
            >
              {passwordMessage.text}
            </p>
          )}
        </Panel>
      </div>
    </div>
  )
}
