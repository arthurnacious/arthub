import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { MatrixRain } from './MatrixRain'

describe('MatrixRain', () => {
  it('renders a decorative, non-interactive canvas', () => {
    const { container } = render(<MatrixRain />)
    const canvas = container.querySelector('canvas.matrix-rain')

    expect(canvas).not.toBeNull()
    expect(canvas).toHaveAttribute('aria-hidden', 'true')
  })
})
