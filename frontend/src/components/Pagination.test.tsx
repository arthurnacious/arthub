import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { Pagination } from './Pagination'

describe('Pagination', () => {
  it('shows the current page, total pages and total count', () => {
    render(<Pagination page={2} pageSize={5} total={23} onPageChange={() => {}} />)

    expect(screen.getByText('2')).toBeInTheDocument()
    expect(screen.getByText(/of 5/)).toBeInTheDocument()
    expect(screen.getByText(/23 total/)).toBeInTheDocument()
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

  it('calls onPageChange with the adjacent page', async () => {
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

    expect(screen.getByText(/of 1/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /next/i })).toBeDisabled()
  })
})
