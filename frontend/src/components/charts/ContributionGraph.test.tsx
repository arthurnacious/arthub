import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { ContributionGraph } from './ContributionGraph'

describe('ContributionGraph', () => {
  it('renders one cell per day in range, aligned to full weeks', () => {
    // Tue 2026-06-02 through Thu 2026-06-04: 3 days, but the grid pads out
    // to full Sun-Sat weeks, so every day in the aligned range gets a cell.
    const { container } = render(
      <ContributionGraph
        data={[
          { date: '2026-06-02', count: 1 },
          { date: '2026-06-03', count: 0 },
          { date: '2026-06-04', count: 5 },
        ]}
      />,
    )

    // 2026-06-02 is a Tuesday, so the aligned week starts Sun 2026-05-31
    // and 2026-06-04 (Thursday) is still within that same week - one
    // 7-cell week, all filled (no null padding beyond the data's own week).
    expect(container.querySelectorAll('[data-testid="contribution-cell"]')).toHaveLength(7)
  })

  it('buckets counts into visually distinct levels', () => {
    const { container } = render(
      <ContributionGraph
        data={[
          { date: '2026-06-02', count: 0 },
          { date: '2026-06-03', count: 8 },
        ]}
      />,
    )

    const cells = container.querySelectorAll('[data-testid="contribution-cell"]')
    const zero = Array.from(cells).find((c) => c.getAttribute('data-date') === '2026-06-02')
    const high = Array.from(cells).find((c) => c.getAttribute('data-date') === '2026-06-03')

    expect(zero?.getAttribute('data-level')).toBe('0')
    expect(high?.getAttribute('data-level')).toBe('4')
  })

  it('includes the date and count in an accessible tooltip', () => {
    const { container } = render(
      <ContributionGraph data={[{ date: '2026-06-02', count: 3 }]} />,
    )

    const cell = container.querySelector('[data-date="2026-06-02"]')
    expect(cell?.querySelector('title')?.textContent).toBe('2026-06-02: 3')
  })

  it('renders nothing for empty data', () => {
    render(<ContributionGraph data={[]} />)

    expect(screen.queryByRole('img')).not.toBeInTheDocument()
  })

  it('labels a month once, at its first week, when the range spans months', () => {
    render(
      <ContributionGraph
        data={[
          { date: '2026-05-28', count: 1 },
          { date: '2026-06-15', count: 1 },
        ]}
      />,
    )

    expect(screen.getByText('May')).toBeInTheDocument()
    expect(screen.getByText('Jun')).toBeInTheDocument()
  })

  it('scales to fill the width of its container', () => {
    const { container } = render(
      <ContributionGraph data={[{ date: '2026-06-02', count: 1 }]} />,
    )

    const svg = container.querySelector('svg')
    expect(svg).toHaveAttribute('viewBox')
    expect(svg?.style.width).toBe('100%')
  })
})
