import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { ActivityGraphPanel } from './ActivityGraphPanel'
import * as activityApi from '../../lib/api/activity'
import { formatLocalDate } from '../../lib/local-date'

function renderPanel() {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={queryClient}>
      <ActivityGraphPanel />
    </QueryClientProvider>,
  )
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('ActivityGraphPanel', () => {
  it('renders a cell for today with the real count from the backend', async () => {
    const today = formatLocalDate(new Date())
    vi.spyOn(activityApi, 'fetchActivityHeatmap').mockResolvedValue([{ date: today, count: 4 }])

    const { container } = renderPanel()
    await screen.findByRole('img')

    const cell = container.querySelector(`[data-date="${today}"]`)
    expect(cell?.querySelector('title')?.textContent).toBe(`${today}: 4`)
  })

  it('fills in the full 371-day window as zero, not just the days the backend returned', async () => {
    const today = formatLocalDate(new Date())
    vi.spyOn(activityApi, 'fetchActivityHeatmap').mockResolvedValue([{ date: today, count: 1 }])

    const { container } = renderPanel()
    await screen.findByRole('img')

    // 371 days padded out to full weeks is at least 371 cells - only
    // today's has real activity, everything else should be a zero.
    const cells = container.querySelectorAll('[data-testid="contribution-cell"]')
    expect(cells.length).toBeGreaterThanOrEqual(371)
    const zeroCells = Array.from(cells).filter((c) => c.getAttribute('data-level') === '0')
    expect(zeroCells.length).toBe(cells.length - 1)
  })

  it('shows an error state with a working retry', async () => {
    const user = userEvent.setup()
    vi.spyOn(activityApi, 'fetchActivityHeatmap')
      .mockRejectedValueOnce(new Error('boom'))
      .mockResolvedValue([])

    renderPanel()
    expect(await screen.findByText("COULDN'T LOAD ACTIVITY GRAPH")).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'retry' }))

    expect(await screen.findByRole('img')).toBeInTheDocument()
  })
})
