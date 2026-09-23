import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { ErrorState } from './ErrorState'

describe('ErrorState', () => {
  it('shows the title and message', () => {
    render(<ErrorState title="COULDN'T LOAD CLIENT" message="Something went wrong." onRetry={() => {}} />)

    expect(screen.getByText("COULDN'T LOAD CLIENT")).toBeInTheDocument()
    expect(screen.getByText('Something went wrong.')).toBeInTheDocument()
  })

  it('calls onRetry when the retry button is clicked', async () => {
    const user = userEvent.setup()
    const onRetry = vi.fn()
    render(<ErrorState title="Error" message="Msg" onRetry={onRetry} />)

    await user.click(screen.getByRole('button', { name: 'retry' }))

    expect(onRetry).toHaveBeenCalledOnce()
  })
})
