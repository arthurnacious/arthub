import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { AsciiFallback } from './AsciiFallback'
import { generateAsciiArt } from '../lib/ascii-art'

describe('AsciiFallback', () => {
  it('renders the seed-derived art, hidden from assistive tech', () => {
    const { container } = render(<AsciiFallback seed="project-1" />)
    const pre = container.querySelector('pre')

    expect(pre).toHaveAttribute('aria-hidden', 'true')
    expect(pre?.textContent).toBe(generateAsciiArt('project-1'))
  })

  it('renders different art for a different seed', () => {
    const a = render(<AsciiFallback seed="project-1" />)
    const b = render(<AsciiFallback seed="project-2" />)

    expect(a.container.querySelector('pre')?.textContent).not.toBe(
      b.container.querySelector('pre')?.textContent,
    )
  })
})
