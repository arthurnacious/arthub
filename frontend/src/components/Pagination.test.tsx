import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { Pagination } from './Pagination'

describe('Pagination', () => {
  it('shows the total count', () => {
    render(<Pagination page={2} pageSize={5} total={23} onPageChange={() => {}} />)

    expect(screen.getByText(/23 total/)).toBeInTheDocument()
  })

  it('renders a button per page when there are few pages', () => {
    render(<Pagination page={2} pageSize={5} total={23} onPageChange={() => {}} />)

    // ceil(23/5) = 5 pages
    for (const page of [1, 2, 3, 4, 5]) {
      expect(screen.getByRole('button', { name: String(page) })).toBeInTheDocument()
    }
  })

  it('marks the current page', () => {
    render(<Pagination page={3} pageSize={5} total={23} onPageChange={() => {}} />)

    expect(screen.getByRole('button', { name: '3' })).toHaveAttribute('aria-current', 'page')
    expect(screen.getByRole('button', { name: '2' })).not.toHaveAttribute('aria-current')
  })

  it('collapses a long page range with ellipsis around the current page', () => {
    render(<Pagination page={10} pageSize={5} total={200} onPageChange={() => {}} />)

    // 40 pages total - only first, last, and a window around current(10)
    // should render as real buttons.
    for (const page of [1, 9, 10, 11, 40]) {
      expect(screen.getByRole('button', { name: String(page) })).toBeInTheDocument()
    }
    expect(screen.queryByRole('button', { name: '20' })).not.toBeInTheDocument()
    expect(screen.getAllByText('...').length).toBeGreaterThan(0)
  })

  it('clicking a page number calls onPageChange with that page', async () => {
    const user = userEvent.setup()
    const onPageChange = vi.fn()
    render(<Pagination page={1} pageSize={5} total={23} onPageChange={onPageChange} />)

    await user.click(screen.getByRole('button', { name: '3' }))
    expect(onPageChange).toHaveBeenCalledWith(3)
  })

  it('disables prev on the first page', () => {
    render(<Pagination page={1} pageSize={5} total={23} onPageChange={() => {}} />)

    expect(screen.getByRole('button', { name: /prev/i })).toBeDisabled()
    expect(screen.getByRole('button', { name: /next/i })).toBeEnabled()
  })

  it('disables next on the last page', () => {
    render(<Pagination page={5} pageSize={5} total={23} onPageChange={() => {}} />)

    expect(screen.getByRole('button', { name: /next/i })).toBeDisabled()
    expect(screen.getByRole('button', { name: /prev/i })).toBeEnabled()
  })

  it('calls onPageChange with the adjacent page from prev/next', async () => {
    const user = userEvent.setup()
    const onPageChange = vi.fn()
    render(<Pagination page={2} pageSize={5} total={23} onPageChange={onPageChange} />)

    await user.click(screen.getByRole('button', { name: /next/i }))
    expect(onPageChange).toHaveBeenCalledWith(3)

    await user.click(screen.getByRole('button', { name: /prev/i }))
    expect(onPageChange).toHaveBeenCalledWith(1)
  })

  it('treats zero results as a single (empty) page', () => {
    render(<Pagination page={1} pageSize={5} total={0} onPageChange={() => {}} />)

    expect(screen.getByRole('button', { name: '1' })).toHaveAttribute('aria-current', 'page')
    expect(screen.getByRole('button', { name: /next/i })).toBeDisabled()
  })
})
