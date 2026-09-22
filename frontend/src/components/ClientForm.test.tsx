import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { ClientForm } from './ClientForm'
import type { ClientFormValues } from './ClientForm'

const EMPTY: ClientFormValues = {
  company: '',
  contact: '',
  email: '',
  status: 'prospect',
}

describe('ClientForm', () => {
  it('renders the given values', () => {
    render(
      <ClientForm
        values={{ company: 'Nimbus Retail', contact: 'Dana', email: 'dana@test', status: 'active' }}
        onChange={() => {}}
        onSubmit={() => {}}
        onCancel={() => {}}
        submitLabel="save"
        submitting={false}
      />,
    )

    expect(screen.getByLabelText(/company/i)).toHaveValue('Nimbus Retail')
    expect(screen.getByLabelText(/contact/i)).toHaveValue('Dana')
    expect(screen.getByLabelText(/email/i)).toHaveValue('dana@test')
  })

  it('calls onChange with the updated values as fields are edited', async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    render(
      <ClientForm
        values={EMPTY}
        onChange={onChange}
        onSubmit={() => {}}
        onCancel={() => {}}
        submitLabel="create"
        submitting={false}
      />,
    )

    await user.type(screen.getByLabelText(/company/i), 'X')

    expect(onChange).toHaveBeenCalledWith({ ...EMPTY, company: 'X' })
  })

  it('calls onSubmit when the form is submitted', async () => {
    const user = userEvent.setup()
    const onSubmit = vi.fn()
    render(
      <ClientForm
        values={EMPTY}
        onChange={() => {}}
        onSubmit={onSubmit}
        onCancel={() => {}}
        submitLabel="create"
        submitting={false}
      />,
    )

    await user.click(screen.getByRole('button', { name: 'create' }))
    expect(onSubmit).toHaveBeenCalledOnce()
  })

  it('calls onCancel when cancel is clicked', async () => {
    const user = userEvent.setup()
    const onCancel = vi.fn()
    render(
      <ClientForm
        values={EMPTY}
        onChange={() => {}}
        onSubmit={() => {}}
        onCancel={onCancel}
        submitLabel="create"
        submitting={false}
      />,
    )

    await user.click(screen.getByRole('button', { name: /cancel/i }))
    expect(onCancel).toHaveBeenCalledOnce()
  })

  it('disables the submit button and shows a busy label while submitting', () => {
    render(
      <ClientForm
        values={EMPTY}
        onChange={() => {}}
        onSubmit={() => {}}
        onCancel={() => {}}
        submitLabel="create"
        submitting
      />,
    )

    const button = screen.getByRole('button', { name: /saving/i })
    expect(button).toBeDisabled()
  })
})
