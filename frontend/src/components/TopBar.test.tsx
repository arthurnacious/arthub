import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { TopBar } from './TopBar'

describe('TopBar', () => {
  it('calls onMenuClick when the menu button is clicked', async () => {
    const user = userEvent.setup()
    const onMenuClick = vi.fn()
    render(<TopBar onMenuClick={onMenuClick} mobileNavOpen={false} />)

    await user.click(screen.getByRole('button', { name: /toggle navigation menu/i }))

    expect(onMenuClick).toHaveBeenCalledOnce()
  })

  it('reflects the open state on the menu button', () => {
    const { rerender } = render(
      <TopBar onMenuClick={() => {}} mobileNavOpen={false} />,
    )
    expect(
      screen.getByRole('button', { name: /toggle navigation menu/i }),
    ).toHaveAttribute('aria-expanded', 'false')

    rerender(<TopBar onMenuClick={() => {}} mobileNavOpen />)
    expect(
      screen.getByRole('button', { name: /toggle navigation menu/i }),
    ).toHaveAttribute('aria-expanded', 'true')
  })
})
