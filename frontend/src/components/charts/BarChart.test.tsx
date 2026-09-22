import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { BarChart } from './BarChart'

describe('BarChart', () => {
  it('renders one bar per data point, with a label each', () => {
    render(
      <BarChart
        data={[
          { label: 'jan', value: 10 },
          { label: 'feb', value: 20 },
        ]}
      />,
    )

    expect(screen.getAllByTestId('bar')).toHaveLength(2)
    expect(screen.getByText('jan')).toBeInTheDocument()
    expect(screen.getByText('feb')).toBeInTheDocument()
  })

  it('scales bar height proportionally to the max value', () => {
    const { container } = render(
      <BarChart
        data={[
          { label: 'a', value: 5 },
          { label: 'b', value: 10 },
        ]}
      />,
    )

    const rects = container.querySelectorAll('[data-testid="bar-rect"]')
    const heightA = Number(rects[0].getAttribute('height'))
    const heightB = Number(rects[1].getAttribute('height'))

    expect(heightB).toBeCloseTo(heightA * 2, 1)
  })

  it('handles an all-zero dataset without NaN heights', () => {
    const { container } = render(
      <BarChart data={[{ label: 'a', value: 0 }]} />,
    )

    const rect = container.querySelector('[data-testid="bar-rect"]')
    expect(Number(rect?.getAttribute('height'))).toBe(0)
  })

  it('renders nothing for empty data', () => {
    render(<BarChart data={[]} />)

    expect(screen.queryByRole('img')).not.toBeInTheDocument()
  })

  it('renders muted bars as a flat gray placeholder, not scaled by value', () => {
    const { container } = render(
      <BarChart
        data={[
          { label: 'jan', value: 20 },
          { label: 'dec', value: 0, muted: true },
        ]}
      />,
    )

    const rects = container.querySelectorAll('[data-testid="bar-rect"]')
    expect(rects[0].className.baseVal).toContain('fill-[var(--accent)]')
    expect(rects[1].className.baseVal).toContain('fill-[var(--border)]')
    expect(Number(rects[1].getAttribute('height'))).toBeGreaterThan(0)
  })

  it('does not let muted bars affect the scale of real bars', () => {
    const { container } = render(
      <BarChart
        data={[
          { label: 'jan', value: 10 },
          { label: 'dec', value: 0, muted: true },
        ]}
      />,
    )

    const rects = container.querySelectorAll('[data-testid="bar-rect"]')
    // jan is the only real value, so it should be at (or near) full chart
    // height, not shrunk by treating the muted 0 as part of the scale.
    expect(Number(rects[0].getAttribute('height'))).toBeGreaterThan(80)
  })

  it('labels each group once, at its first bar', () => {
    render(
      <BarChart
        data={[
          { label: 'oct', value: 10, group: '2025' },
          { label: 'nov', value: 10, group: '2025' },
          { label: 'dec', value: 10, group: '2025' },
          { label: 'jan', value: 10, group: '2026' },
          { label: 'feb', value: 10, group: '2026' },
        ]}
      />,
    )

    expect(screen.getAllByText('2025')).toHaveLength(1)
    expect(screen.getAllByText('2026')).toHaveLength(1)
  })

  it('renders no group labels when group is omitted', () => {
    const { container } = render(
      <BarChart data={[{ label: 'jan', value: 10 }]} />,
    )

    expect(container.querySelector('[data-testid="group-label"]')).toBeNull()
  })

  it('draws a vertical divider between groups, but not before the first', () => {
    const { container } = render(
      <BarChart
        data={[
          { label: 'nov', value: 10, group: '2025' },
          { label: 'dec', value: 10, group: '2025' },
          { label: 'jan', value: 10, group: '2026' },
          { label: 'feb', value: 10, group: '2026' },
          { label: 'mar', value: 10, group: '2027' },
        ]}
      />,
    )

    // 3 groups -> 2 dividers (between 1st/2nd, and 2nd/3rd - none before
    // the very first group).
    expect(container.querySelectorAll('[data-testid="group-divider"]')).toHaveLength(2)
  })

  it('draws one border under the whole year-label row, spanning the full width', () => {
    const { container } = render(
      <BarChart
        data={[
          { label: 'nov', value: 10, group: '2025' },
          { label: 'dec', value: 10, group: '2025' },
          { label: 'jan', value: 10, group: '2026' },
        ]}
      />,
    )

    const rules = container.querySelectorAll('[data-testid="group-row-border"]')
    expect(rules).toHaveLength(1)

    const x1 = Number(rules[0].getAttribute('x1'))
    const x2 = Number(rules[0].getAttribute('x2'))
    expect(x1).toBe(0)
    expect(x2).toBe(3 * (24 + 16)) // data.length * (BAR_WIDTH + BAR_GAP)
  })

  it('renders no dividers/row border when group is omitted', () => {
    const { container } = render(
      <BarChart data={[{ label: 'jan', value: 10 }]} />,
    )

    expect(container.querySelector('[data-testid="group-divider"]')).toBeNull()
    expect(container.querySelector('[data-testid="group-row-border"]')).toBeNull()
  })
})
