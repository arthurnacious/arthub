import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'
import { NotificationsMenu } from './NotificationsMenu'

describe('NotificationsMenu', () => {
  it('shows the unread count on the toggle button', () => {
    render(<NotificationsMenu />)

    expect(
      screen.getByRole('button', { name: /notifications \(2 unread\)/i }),
    ).toBeInTheDocument()
  })

  it('is closed until clicked', () => {
    render(<NotificationsMenu />)

    expect(screen.queryByText('notifications')).not.toBeInTheDocument()
  })

  it('opens the panel and lists notifications on click', async () => {
    const user = userEvent.setup()
    render(<NotificationsMenu />)

    await user.click(screen.getByRole('button', { name: /notifications/i }))

    expect(
      screen.getByText('quote Q-1042 was viewed by the client'),
    ).toBeInTheDocument()
  })

  it('closes on Escape', async () => {
    const user = userEvent.setup()
    render(<NotificationsMenu />)

    await user.click(screen.getByRole('button', { name: /notifications/i }))
    expect(screen.getByText('notifications')).toBeInTheDocument()

    fireEvent.keyDown(document, { key: 'Escape' })

    expect(screen.queryByText('notifications')).not.toBeInTheDocument()
  })
})
