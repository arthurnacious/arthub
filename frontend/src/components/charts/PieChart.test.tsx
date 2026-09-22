import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { PieChart } from './PieChart'

describe('PieChart', () => {
  it('renders one slice per data point', () => {
    render(
      <PieChart
        data={[
          { label: 'paid', value: 3, tone: 'success' },
          { label: 'overdue', value: 1, tone: 'danger' },
        ]}
      />,
    )

    expect(screen.getAllByTestId('pie-slice')).toHaveLength(2)
  })

  it('shows each label with its correct percentage in the legend', () => {
    render(
      <PieChart
        data={[
          { label: 'paid', value: 3 },
          { label: 'overdue', value: 1 },
        ]}
      />,
    )

    expect(screen.getByText('paid')).toBeInTheDocument()
    expect(screen.getByText('75%')).toBeInTheDocument()
    expect(screen.getByText('overdue')).toBeInTheDocument()
    expect(screen.getByText('25%')).toBeInTheDocument()
  })

  it('gives the largest slice a proportionally longer dasharray', () => {
    const { container } = render(
      <PieChart
        data={[
          { label: 'small', value: 1 },
          { label: 'big', value: 3 },
        ]}
      />,
    )

    const slices = container.querySelectorAll('[data-testid="pie-slice"]')
    const smallLength = Number(slices[0].getAttribute('stroke-dasharray')?.split(' ')[0])
    const bigLength = Number(slices[1].getAttribute('stroke-dasharray')?.split(' ')[0])

    expect(bigLength).toBeCloseTo(smallLength * 3, 1)
  })

  it('handles an all-zero dataset without NaN percentages', () => {
    render(<PieChart data={[{ label: 'nothing', value: 0 }]} />)

    expect(screen.getByText('0%')).toBeInTheDocument()
  })
})
