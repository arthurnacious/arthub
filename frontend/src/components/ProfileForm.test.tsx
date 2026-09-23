import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { ProfileForm } from './ProfileForm'

const VALUES = { name: 'Dana', email: 'dana@example.com' }

describe('ProfileForm', () => {
  it('shows the current name and email', () => {
    render(<ProfileForm values={VALUES} onChange={() => {}} onSubmit={() => {}} submitting={false} />)

    expect(screen.getByLabelText('name')).toHaveValue('Dana')
    expect(screen.getByLabelText('email')).toHaveValue('dana@example.com')
  })

  it('calls onChange as fields are edited', async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    render(<ProfileForm values={VALUES} onChange={onChange} onSubmit={() => {}} submitting={false} />)

    await user.type(screen.getByLabelText('name'), 'X')

    expect(onChange).toHaveBeenCalledWith({ ...VALUES, name: 'DanaX' })
  })

  it('calls onSubmit when saved', async () => {
    const user = userEvent.setup()
    const onSubmit = vi.fn()
    render(<ProfileForm values={VALUES} onChange={() => {}} onSubmit={onSubmit} submitting={false} />)

    await user.click(screen.getByRole('button', { name: 'save' }))

    expect(onSubmit).toHaveBeenCalledOnce()
  })

  it('disables save while empty', () => {
    render(
      <ProfileForm
        values={{ name: '', email: '' }}
        onChange={() => {}}
        onSubmit={() => {}}
        submitting={false}
      />,
    )

    expect(screen.getByRole('button', { name: 'save' })).toBeDisabled()
  })
})
