import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { ConfirmDialog } from './ConfirmDialog'

describe('ConfirmDialog', () => {
  it('renders the title and message', () => {
    render(
      <ConfirmDialog
        title="delete client"
        message="Delete Nimbus Retail? This can't be undone."
        onConfirm={() => {}}
        onCancel={() => {}}
      />,
    )

    expect(screen.getByText('delete client')).toBeInTheDocument()
    expect(
      screen.getByText("Delete Nimbus Retail? This can't be undone."),
    ).toBeInTheDocument()
  })

  it('calls onConfirm when the confirm button is clicked', async () => {
    const user = userEvent.setup()
    const onConfirm = vi.fn()
    render(<ConfirmDialog message="sure?" onConfirm={onConfirm} onCancel={() => {}} />)

    await user.click(screen.getByRole('button', { name: /confirm/i }))
    expect(onConfirm).toHaveBeenCalledOnce()
  })

  it('calls onCancel when the cancel button is clicked', async () => {
    const user = userEvent.setup()
    const onCancel = vi.fn()
    render(<ConfirmDialog message="sure?" onConfirm={() => {}} onCancel={onCancel} />)

    await user.click(screen.getByRole('button', { name: /cancel/i }))
    expect(onCancel).toHaveBeenCalledOnce()
  })

  it('calls onCancel on Escape', () => {
    const onCancel = vi.fn()
    render(<ConfirmDialog message="sure?" onConfirm={() => {}} onCancel={onCancel} />)

    fireEvent.keyDown(document, { key: 'Escape' })
    expect(onCancel).toHaveBeenCalledOnce()
  })

  it('uses custom labels when given', () => {
    render(
      <ConfirmDialog
        message="sure?"
        confirmLabel="delete"
        cancelLabel="keep"
        onConfirm={() => {}}
        onCancel={() => {}}
      />,
    )

    expect(screen.getByRole('button', { name: 'delete' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'keep' })).toBeInTheDocument()
  })
})
