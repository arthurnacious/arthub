import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it } from 'vitest'
import { ThemeToggle } from './ThemeToggle'
import { themeStore } from '../stores/theme-store'

describe('ThemeToggle', () => {
  beforeEach(() => {
    themeStore.setColor('green')
  })

  it('shows the current theme color on the trigger', () => {
    render(<ThemeToggle />)

    expect(screen.getByRole('button', { name: /green/i })).toBeInTheDocument()
  })

  it('is closed until clicked', () => {
    render(<ThemeToggle />)

    expect(screen.queryByRole('button', { name: /^purple$/i })).not.toBeInTheDocument()
  })

  it('lists every color option when opened', async () => {
    const user = userEvent.setup()
    render(<ThemeToggle />)

    await user.click(screen.getByRole('button', { name: /theme color: green/i }))

    for (const color of ['green', 'purple', 'orange', 'red']) {
      expect(screen.getByRole('button', { name: new RegExp(`^${color}`, 'i') })).toBeInTheDocument()
    }
  })

  it('selecting a color sets it and closes the dropdown', async () => {
    const user = userEvent.setup()
    render(<ThemeToggle />)

    await user.click(screen.getByRole('button', { name: /theme color: green/i }))
    await user.click(screen.getByRole('button', { name: /^purple$/i }))

    expect(themeStore.store.state).toBe('purple')
    expect(screen.getByRole('button', { name: /theme color: purple/i })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /^orange$/i })).not.toBeInTheDocument()
  })

  it('closes on Escape', async () => {
    const user = userEvent.setup()
    render(<ThemeToggle />)

    await user.click(screen.getByRole('button', { name: /theme color: green/i }))
    fireEvent.keyDown(document, { key: 'Escape' })

    expect(screen.queryByRole('button', { name: /^purple$/i })).not.toBeInTheDocument()
  })
})
