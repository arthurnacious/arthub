import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { KernelLog } from './KernelLog'

describe('KernelLog', () => {
  it('renders each entry with its timestamp and message', () => {
    render(
      <KernelLog
        entries={[
          { id: '1', timestamp: '10:00:00', message: 'client created' },
          { id: '2', timestamp: '10:01:00', message: 'quote sent', level: 'warn' },
        ]}
      />,
    )

    expect(screen.getByText('[10:00:00]')).toBeInTheDocument()
    expect(screen.getByText('client created')).toBeInTheDocument()
    expect(screen.getByText('quote sent')).toBeInTheDocument()
  })

  it('shows a placeholder when there are no entries', () => {
    render(<KernelLog entries={[]} />)

    expect(screen.getByText('-- no entries --')).toBeInTheDocument()
  })
})
