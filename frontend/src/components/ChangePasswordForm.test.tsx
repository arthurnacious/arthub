import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { ChangePasswordForm } from './ChangePasswordForm'

async function fillAndSubmit(
  user: ReturnType<typeof userEvent.setup>,
  { current, next, confirm }: { current: string; next: string; confirm: string },
) {
  await user.type(screen.getByLabelText('current password'), current)
  await user.type(screen.getByLabelText('new password'), next)
  await user.type(screen.getByLabelText('confirm new password'), confirm)
  await user.click(screen.getByRole('button', { name: 'change password' }))
}

describe('ChangePasswordForm', () => {
  it('submits current/new password once everything is valid', async () => {
    const user = userEvent.setup()
    const onSubmit = vi.fn()
    render(<ChangePasswordForm onSubmit={onSubmit} submitting={false} />)

    await fillAndSubmit(user, { current: 'old-password', next: 'new-password', confirm: 'new-password' })

    expect(onSubmit).toHaveBeenCalledWith({ currentPassword: 'old-password', newPassword: 'new-password' })
  })

  it('disables submit and shows an error when the new password is too short', async () => {
    const user = userEvent.setup()
    const onSubmit = vi.fn()
    render(<ChangePasswordForm onSubmit={onSubmit} submitting={false} />)

    await user.type(screen.getByLabelText('current password'), 'old-password')
    await user.type(screen.getByLabelText('new password'), 'short')

    expect(screen.getByText('must be at least 8 characters')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'change password' })).toBeDisabled()
  })

  it('disables submit and shows an error when the confirmation does not match', async () => {
    const user = userEvent.setup()
    render(<ChangePasswordForm onSubmit={() => {}} submitting={false} />)

    await user.type(screen.getByLabelText('current password'), 'old-password')
    await user.type(screen.getByLabelText('new password'), 'new-password')
    await user.type(screen.getByLabelText('confirm new password'), 'different')

    expect(screen.getByText("passwords don't match")).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'change password' })).toBeDisabled()
  })

  it('disables submit until every field is filled', () => {
    render(<ChangePasswordForm onSubmit={() => {}} submitting={false} />)

    expect(screen.getByRole('button', { name: 'change password' })).toBeDisabled()
  })
})
