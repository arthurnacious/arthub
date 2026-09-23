import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { Skeleton } from './Skeleton'

describe('Skeleton', () => {
  it('renders a pulsing, decorative block', () => {
    const { container } = render(<Skeleton className="h-4 w-24" />)
    const el = container.firstElementChild

    expect(el).toHaveAttribute('aria-hidden', 'true')
    expect(el).toHaveClass('animate-pulse', 'h-4', 'w-24')
  })
})
