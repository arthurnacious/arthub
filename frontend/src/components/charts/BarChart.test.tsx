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
})
