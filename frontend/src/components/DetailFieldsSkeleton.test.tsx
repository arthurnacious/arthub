import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { DetailFieldsSkeleton } from './DetailFieldsSkeleton'

describe('DetailFieldsSkeleton', () => {
  it('renders the requested number of field rows', () => {
    const { container } = render(<DetailFieldsSkeleton rows={3} />)

    expect(container.firstElementChild?.children).toHaveLength(3)
  })

  it('defaults to 4 rows', () => {
    const { container } = render(<DetailFieldsSkeleton />)

    expect(container.firstElementChild?.children).toHaveLength(4)
  })
})
