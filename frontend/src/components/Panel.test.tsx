import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { Panel } from './Panel'

describe('Panel', () => {
  it('renders the title and children', () => {
    const { container } = render(
      <Panel title="status">
        <p>panel body</p>
      </Panel>,
    )

    expect(container.querySelector('header')?.textContent).toBe('status')
    expect(screen.getByText('panel body')).toBeInTheDocument()
  })
})
