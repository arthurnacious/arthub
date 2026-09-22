import { fireEvent, render } from '@testing-library/react'
import { useRef } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { useDismissable } from './use-dismissable'

function Harness({ open, onDismiss }: { open: boolean; onDismiss: () => void }) {
  const ref = useRef<HTMLDivElement>(null)
  useDismissable(ref, open, onDismiss)

  return (
    <div>
      <div ref={ref} data-testid="inside">
        inside
      </div>
      <div data-testid="outside">outside</div>
    </div>
  )
}

describe('useDismissable', () => {
  it('calls onDismiss on an outside click', () => {
    const onDismiss = vi.fn()
    const { getByTestId } = render(<Harness open onDismiss={onDismiss} />)

    fireEvent.mouseDown(getByTestId('outside'))

    expect(onDismiss).toHaveBeenCalledOnce()
  })

  it('does not call onDismiss on an inside click', () => {
    const onDismiss = vi.fn()
    const { getByTestId } = render(<Harness open onDismiss={onDismiss} />)

    fireEvent.mouseDown(getByTestId('inside'))

    expect(onDismiss).not.toHaveBeenCalled()
  })

  it('calls onDismiss on Escape', () => {
    const onDismiss = vi.fn()
    render(<Harness open onDismiss={onDismiss} />)

    fireEvent.keyDown(document, { key: 'Escape' })

    expect(onDismiss).toHaveBeenCalledOnce()
  })

  it('does nothing while closed', () => {
    const onDismiss = vi.fn()
    const { getByTestId } = render(<Harness open={false} onDismiss={onDismiss} />)

    fireEvent.mouseDown(getByTestId('outside'))
    fireEvent.keyDown(document, { key: 'Escape' })

    expect(onDismiss).not.toHaveBeenCalled()
  })
})
