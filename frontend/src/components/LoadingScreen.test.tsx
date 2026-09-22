import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { LoadingScreen } from './LoadingScreen'

describe('LoadingScreen', () => {
  it('renders the logo and a booting message', () => {
    render(<LoadingScreen />)

    expect(screen.getByText('ART')).toBeInTheDocument()
    expect(screen.getByText('HUB')).toBeInTheDocument()
    expect(screen.getByText('booting...')).toBeInTheDocument()
  })

  it('uses the monochrome logo, not the colored pill', () => {
    render(<LoadingScreen />)

    expect(screen.getByText('HUB').className).not.toContain('bg-[var(--accent)]')
  })
})
