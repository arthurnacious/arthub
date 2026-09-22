import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { RowActionsMenu } from './RowActionsMenu'

describe('RowActionsMenu', () => {
  it('is closed until the trigger is clicked', () => {
    render(
      <RowActionsMenu
        item="row-1"
        label="row-1"
        onView={() => {}}
        onUpdate={() => {}}
        onDelete={() => {}}
      />,
    )

    expect(screen.queryByRole('button', { name: /^view$/i })).not.toBeInTheDocument()
  })

  it('calls onView with the item and closes the menu', async () => {
    const user = userEvent.setup()
    const onView = vi.fn()
    render(
      <RowActionsMenu
        item="row-1"
        label="row-1"
        onView={onView}
        onUpdate={() => {}}
        onDelete={() => {}}
      />,
    )

    await user.click(screen.getByRole('button', { name: /actions for row-1/i }))
    await user.click(screen.getByRole('button', { name: /^view$/i }))

    expect(onView).toHaveBeenCalledWith('row-1')
    expect(screen.queryByRole('button', { name: /^view$/i })).not.toBeInTheDocument()
  })

  it('calls onUpdate with the item', async () => {
    const user = userEvent.setup()
    const onUpdate = vi.fn()
    render(
      <RowActionsMenu
        item="row-1"
        label="row-1"
        onView={() => {}}
        onUpdate={onUpdate}
        onDelete={() => {}}
      />,
    )

    await user.click(screen.getByRole('button', { name: /actions for row-1/i }))
    await user.click(screen.getByRole('button', { name: /^update$/i }))

    expect(onUpdate).toHaveBeenCalledWith('row-1')
  })

  it('calls onDelete with the item', async () => {
    const user = userEvent.setup()
    const onDelete = vi.fn()
    render(
      <RowActionsMenu
        item="row-1"
        label="row-1"
        onView={() => {}}
        onUpdate={() => {}}
        onDelete={onDelete}
      />,
    )

    await user.click(screen.getByRole('button', { name: /actions for row-1/i }))
    await user.click(screen.getByRole('button', { name: /^delete$/i }))

    expect(onDelete).toHaveBeenCalledWith('row-1')
  })
})
