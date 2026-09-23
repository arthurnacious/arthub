import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { QuoteLineItemsEditor } from './QuoteLineItemsEditor'
import type { QuoteLineInput } from '../lib/api/quotes'

const LINES: QuoteLineInput[] = [
  { description: 'Design', quantity: 2, unitPrice: 100, discountType: null, discountValue: 0 },
]

describe('QuoteLineItemsEditor', () => {
  it('shows a placeholder when there are no lines', () => {
    render(<QuoteLineItemsEditor lines={[]} onChange={() => {}} />)

    expect(screen.getByText('-- no line items yet --')).toBeInTheDocument()
  })

  it('shows the computed total for each line', () => {
    render(<QuoteLineItemsEditor lines={LINES} onChange={() => {}} />)

    expect(screen.getByText('200.00')).toBeInTheDocument()
  })

  it('adds an empty line when "add line" is clicked', async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    render(<QuoteLineItemsEditor lines={LINES} onChange={onChange} />)

    await user.click(screen.getByRole('button', { name: '+ add line' }))

    expect(onChange).toHaveBeenCalledWith([
      ...LINES,
      { description: '', quantity: 1, unitPrice: 0, discountType: null, discountValue: 0 },
    ])
  })

  it('removes a line when its remove button is clicked', async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    render(<QuoteLineItemsEditor lines={LINES} onChange={onChange} />)

    await user.click(screen.getByRole('button', { name: 'Remove line 1' }))

    expect(onChange).toHaveBeenCalledWith([])
  })

  it('updates a line field without touching the others', () => {
    const onChange = vi.fn()
    const twoLines = [...LINES, { description: 'Dev', quantity: 1, unitPrice: 50, discountType: null, discountValue: 0 }]
    render(<QuoteLineItemsEditor lines={twoLines} onChange={onChange} />)

    fireEvent.change(screen.getByLabelText('Line 1 description'), { target: { value: 'X' } })

    expect(onChange).toHaveBeenCalledWith([{ ...twoLines[0], description: 'X' }, twoLines[1]])
  })

  it('disables the discount value input until a discount type is chosen', () => {
    render(<QuoteLineItemsEditor lines={LINES} onChange={() => {}} />)

    expect(screen.getByLabelText('Line 1 discount value')).toBeDisabled()
  })
})
