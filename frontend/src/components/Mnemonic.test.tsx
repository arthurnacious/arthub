import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { Mnemonic } from './Mnemonic'

describe('Mnemonic', () => {
  it('splits the first character out from the rest', () => {
    const { container } = render(<Mnemonic text="clock" />)
    const spans = container.querySelectorAll('span span')

    expect(spans[0].textContent).toBe('c')
    expect(spans[1].textContent).toBe('lock')
  })

  it('renders nothing for an empty string', () => {
    const { container } = render(<Mnemonic text="" />)

    expect(container).toBeEmptyDOMElement()
  })

  it('handles a single character', () => {
    const { container } = render(<Mnemonic text="q" />)
    const spans = container.querySelectorAll('span span')

    expect(spans[0].textContent).toBe('q')
    expect(spans[1].textContent).toBe('')
  })
})
