import { act, fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { AuditLogPanel } from './AuditLogPanel'
import * as activityApi from '../../lib/api/activity'

function renderPanel(props: Partial<Parameters<typeof AuditLogPanel>[0]> = {}) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  const onPageChange = vi.fn()
  const onSearchChange = vi.fn()

  const result = render(
    <QueryClientProvider client={queryClient}>
      <AuditLogPanel
        page={1}
        q=""
        onPageChange={onPageChange}
        onSearchChange={onSearchChange}
        {...props}
      />
    </QueryClientProvider>,
  )

  return { ...result, onPageChange, onSearchChange }
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('AuditLogPanel', () => {
  it('renders entries once loaded', async () => {
    vi.spyOn(activityApi, 'fetchActivityLog').mockResolvedValue({
      data: [
        {
          id: '1',
          user_id: null,
          action: 'client.created',
          description: 'Created client Acme',
          subject_type: 'client',
          subject_id: 'abc',
          metadata: null,
          created_at: new Date().toISOString(),
        },
      ],
      page: 1,
      pageSize: 5,
      total: 1,
    })

    renderPanel()

    expect(await screen.findByText('Created client Acme')).toBeInTheDocument()
  })

  it('shows an error message without crashing when the request fails', async () => {
    vi.spyOn(activityApi, 'fetchActivityLog').mockRejectedValue(new Error('boom'))

    renderPanel()

    expect(await screen.findByText("COULDN'T LOAD ACTIVITY")).toBeInTheDocument()
  })

  it('retries the query when the retry button is clicked', async () => {
    const user = userEvent.setup()
    vi.spyOn(activityApi, 'fetchActivityLog')
      .mockRejectedValueOnce(new Error('boom'))
      .mockResolvedValue({ data: [], page: 1, pageSize: 5, total: 0 })

    renderPanel()
    await user.click(await screen.findByRole('button', { name: 'retry' }))

    expect(await screen.findByText('-- no activity yet --')).toBeInTheDocument()
  })

  it('calls onPageChange when a pagination control is clicked', async () => {
    vi.spyOn(activityApi, 'fetchActivityLog').mockResolvedValue({
      data: [],
      page: 1,
      pageSize: 5,
      total: 12,
    })
    const user = userEvent.setup()

    const { onPageChange } = renderPanel()

    await user.click(await screen.findByRole('button', { name: '2' }))

    expect(onPageChange).toHaveBeenCalledWith(2)
  })

  describe('search debounce', () => {
    beforeEach(() => {
      vi.useFakeTimers()
    })

    afterEach(() => {
      vi.useRealTimers()
    })

    it('calls onSearchChange only after typing settles', async () => {
      vi.spyOn(activityApi, 'fetchActivityLog').mockResolvedValue({
        data: [],
        page: 1,
        pageSize: 5,
        total: 0,
      })
      const { onSearchChange } = renderPanel()

      fireEvent.change(
        screen.getByPlaceholderText(/search action or description/i),
        { target: { value: 'nimbus' } },
      )

      expect(onSearchChange).not.toHaveBeenCalled()

      await act(async () => {
        vi.advanceTimersByTime(300)
      })

      expect(onSearchChange).toHaveBeenCalledWith('nimbus')
    })
  })
})
