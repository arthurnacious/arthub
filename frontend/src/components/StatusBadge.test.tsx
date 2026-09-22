import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { StatusBadge } from './StatusBadge'

describe('StatusBadge', () => {
  it('renders the given label', () => {
    render(<StatusBadge label="paid" tone="success" />)

    expect(screen.getByText('paid')).toBeInTheDocument()
  })

  it('defaults to the neutral tone', () => {
    render(<StatusBadge label="draft" />)

    expect(screen.getByText('draft').className).toContain('zinc')
  })
})
