import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'
import { NotificationsMenu } from './NotificationsMenu'
import type { Notification } from './NotificationsMenu'

const SEEDED: Notification[] = [
  { id: '1', message: 'Invoice INV-0001 was paid', timestamp: '2m ago' },
  { id: '2', message: 'Invoice INV-0002 was paid', timestamp: '1h ago' },
]

describe('NotificationsMenu', () => {
  it('is closed until clicked', () => {
    render(<NotificationsMenu />)

    expect(screen.queryByText('notifications')).not.toBeInTheDocument()
  })

  it('shows no badge and an "all caught up" placeholder - there are no real notifications yet', async () => {
    const user = userEvent.setup()
    render(<NotificationsMenu />)

    expect(screen.getByRole('button', { name: /notifications/i })).toBeInTheDocument()
    expect(screen.queryByText(/^\d+$/)).not.toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: /notifications/i }))

    expect(screen.getByText('-- all caught up --')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /clear all/i })).not.toBeInTheDocument()
  })

  it('shows the count badge and lists seeded notifications', async () => {
    const user = userEvent.setup()
    render(<NotificationsMenu initialNotifications={SEEDED} />)

    expect(screen.getByRole('button', { name: /notifications/i })).toHaveTextContent('2')

    await user.click(screen.getByRole('button', { name: /notifications/i }))

    expect(screen.getByText('Invoice INV-0001 was paid')).toBeInTheDocument()
    expect(screen.getByText('Invoice INV-0002 was paid')).toBeInTheDocument()
  })

  it('removes a single notification without touching the others', async () => {
    const user = userEvent.setup()
    render(<NotificationsMenu initialNotifications={SEEDED} />)

    await user.click(screen.getByRole('button', { name: /notifications/i }))
    await user.click(screen.getByRole('button', { name: /dismiss invoice inv-0001/i }))

    expect(screen.queryByText('Invoice INV-0001 was paid')).not.toBeInTheDocument()
    expect(screen.getByText('Invoice INV-0002 was paid')).toBeInTheDocument()
  })

  it('clears all notifications and shows the empty state', async () => {
    const user = userEvent.setup()
    render(<NotificationsMenu initialNotifications={SEEDED} />)

    await user.click(screen.getByRole('button', { name: /notifications/i }))
    await user.click(screen.getByRole('button', { name: /clear all/i }))

    expect(screen.getByText('-- all caught up --')).toBeInTheDocument()
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
