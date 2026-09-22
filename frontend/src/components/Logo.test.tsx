import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { Logo } from './Logo'

describe('Logo', () => {
  it('renders ART and HUB', () => {
    render(<Logo />)

    expect(screen.getByText('ART')).toBeInTheDocument()
    expect(screen.getByText('HUB')).toBeInTheDocument()
  })

  it('gives HUB a colored pill background by default', () => {
    render(<Logo />)

    expect(screen.getByText('HUB').className).toContain('bg-[var(--accent)]')
  })

  it('drops the colored pill and blinking cursor in monochrome mode', () => {
    const { container } = render(<Logo monochrome />)

    expect(screen.getByText('HUB').className).not.toContain('bg-[var(--accent)]')
    expect(container.querySelector('.cursor-blink')).toBeNull()
  })
})
