import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { ScanlineOverlay } from './ScanlineOverlay'

describe('ScanlineOverlay', () => {
  it('renders as a decorative, non-interactive layer', () => {
    const { container } = render(<ScanlineOverlay />)
    const overlay = container.querySelector('.scanline-overlay')

    expect(overlay).not.toBeNull()
    expect(overlay).toHaveAttribute('aria-hidden', 'true')
  })
})
