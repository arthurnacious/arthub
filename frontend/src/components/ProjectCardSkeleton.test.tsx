import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { ProjectCardSkeleton } from './ProjectCardSkeleton'

describe('ProjectCardSkeleton', () => {
  it('renders a decorative placeholder card', () => {
    const { container } = render(<ProjectCardSkeleton />)

    expect(container.querySelectorAll('[aria-hidden="true"]').length).toBeGreaterThan(0)
  })
})
