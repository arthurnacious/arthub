import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { TableRowsSkeleton } from './TableRowsSkeleton'

describe('TableRowsSkeleton', () => {
  it('renders the requested number of rows, each with one skeleton bar per column', () => {
    const { container } = render(
      <table>
        <tbody>
          <TableRowsSkeleton
            rows={3}
            columns={[
              { className: 'py-2', width: 'w-32' },
              { className: 'py-2', width: 'w-16', align: 'right' },
            ]}
          />
        </tbody>
      </table>,
    )

    expect(container.querySelectorAll('tr')).toHaveLength(3)
    expect(container.querySelectorAll('td')).toHaveLength(6)
    expect(container.querySelectorAll('[aria-hidden="true"]')).toHaveLength(6)
  })

  it('defaults to 5 rows', () => {
    const { container } = render(
      <table>
        <tbody>
          <TableRowsSkeleton columns={[{ className: '', width: 'w-32' }]} />
        </tbody>
      </table>,
    )

    expect(container.querySelectorAll('tr')).toHaveLength(5)
  })
})
