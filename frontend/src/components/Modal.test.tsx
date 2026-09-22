import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { Modal } from './Modal'

describe('Modal', () => {
  it('renders its title and children', () => {
    render(
      <Modal title="new client" onClose={() => {}}>
        <p>form goes here</p>
      </Modal>,
    )

    expect(screen.getByText('new client')).toBeInTheDocument()
    expect(screen.getByText('form goes here')).toBeInTheDocument()
  })

  it('calls onClose when the close button is clicked', async () => {
    const user = userEvent.setup()
    const onClose = vi.fn()
    render(
      <Modal title="new client" onClose={onClose}>
        content
      </Modal>,
    )

    await user.click(screen.getByRole('button', { name: /close/i }))
    expect(onClose).toHaveBeenCalledOnce()
  })

  it('calls onClose on Escape', () => {
    const onClose = vi.fn()
    render(
      <Modal title="new client" onClose={onClose}>
        content
      </Modal>,
    )

    fireEvent.keyDown(document, { key: 'Escape' })
    expect(onClose).toHaveBeenCalledOnce()
  })

  it('calls onClose on an outside (backdrop) click', () => {
    const onClose = vi.fn()
    const { container } = render(
      <Modal title="new client" onClose={onClose}>
        content
      </Modal>,
    )

    fireEvent.mouseDown(container.firstChild as Element)
    expect(onClose).toHaveBeenCalledOnce()
  })
})
