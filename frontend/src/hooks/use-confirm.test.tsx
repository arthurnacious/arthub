import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'
import { useConfirm } from './use-confirm'

function Harness({ onResult }: { onResult: (value: boolean) => void }) {
  const { confirm, dialog } = useConfirm()

  return (
    <div>
      <button
        type="button"
        onClick={async () => {
          const result = await confirm({ message: 'Delete this client?' })
          onResult(result)
        }}
      >
        trigger
      </button>
      {dialog}
    </div>
  )
}

describe('useConfirm', () => {
  it('shows no dialog until confirm() is called', () => {
    render(<Harness onResult={() => {}} />)

    expect(screen.queryByText('Delete this client?')).not.toBeInTheDocument()
  })

  it('resolves true and hides the dialog when confirmed', async () => {
    const user = userEvent.setup()
    const results: boolean[] = []
    render(<Harness onResult={(v) => results.push(v)} />)

    await user.click(screen.getByRole('button', { name: 'trigger' }))
    expect(screen.getByText('Delete this client?')).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: /^confirm$/i }))

    expect(results).toEqual([true])
    expect(screen.queryByText('Delete this client?')).not.toBeInTheDocument()
  })

  it('resolves false and hides the dialog when cancelled', async () => {
    const user = userEvent.setup()
    const results: boolean[] = []
    render(<Harness onResult={(v) => results.push(v)} />)

    await user.click(screen.getByRole('button', { name: 'trigger' }))
    await user.click(screen.getByRole('button', { name: /cancel/i }))

    expect(results).toEqual([false])
    expect(screen.queryByText('Delete this client?')).not.toBeInTheDocument()
  })
})
