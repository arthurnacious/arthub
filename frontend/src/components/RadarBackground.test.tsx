import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { RadarBackground } from './RadarBackground'

describe('RadarBackground', () => {
  it('renders as a decorative, non-interactive layer', () => {
    const { container } = render(<RadarBackground />)
    const root = container.querySelector('.radar-bg')

    expect(root).not.toBeNull()
    expect(root).toHaveAttribute('aria-hidden', 'true')
  })

  it('renders the sweep, the rings and a handful of blips', () => {
    const { container } = render(<RadarBackground />)

    expect(container.querySelector('.radar-sweep')).not.toBeNull()
    expect(container.querySelector('.radar-rings')).not.toBeNull()
    expect(container.querySelectorAll('.radar-blip')).toHaveLength(9)
  })

  it('randomizes blip positions on each mount', () => {
    const { container: first } = render(<RadarBackground />)
    const { container: second } = render(<RadarBackground />)

    const firstTops = Array.from(first.querySelectorAll('.radar-blip')).map(
      (el) => (el as HTMLElement).style.top,
    )
    const secondTops = Array.from(second.querySelectorAll('.radar-blip')).map(
      (el) => (el as HTMLElement).style.top,
    )

    expect(firstTops).not.toEqual(secondTops)
  })

  it('keeps every blip on the same animation clock as the sweep, so each only flashes once the sweep reaches its angle', () => {
    const { container } = render(<RadarBackground />)

    const sweepDuration = (
      container.querySelector('.radar-sweep') as HTMLElement
    ).style.animationDuration
    expect(sweepDuration).toMatch(/^\d+s$/)

    const sweepSeconds = Number.parseFloat(sweepDuration)

    for (const blip of container.querySelectorAll('.radar-blip')) {
      const style = (blip as HTMLElement).style
      expect(style.animationDuration).toBe(sweepDuration)

      const delaySeconds = Number.parseFloat(style.animationDelay)
      expect(delaySeconds).toBeGreaterThanOrEqual(0)
      expect(delaySeconds).toBeLessThan(sweepSeconds)
    }
  })
})
