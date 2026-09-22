import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it } from 'vitest'
import { ThemeToggle } from './ThemeToggle'
import { themeStore } from '../stores/theme-store'

describe('ThemeToggle', () => {
  beforeEach(() => {
    themeStore.setColor('green')
  })

  it('shows the current theme color', () => {
    render(<ThemeToggle />)

    expect(screen.getByRole('button', { name: /green/i })).toBeInTheDocument()
  })

  it('cycles to the next color on click', async () => {
    const user = userEvent.setup()
    render(<ThemeToggle />)

    await user.click(screen.getByRole('button', { name: /green/i }))

    expect(screen.getByRole('button', { name: /purple/i })).toBeInTheDocument()
  })
})
