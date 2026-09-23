import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { ProjectsStatCard } from './ProjectsStatCard'
import * as projectsApi from '../../lib/api/projects'

function renderCard() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <ul>
        <ProjectsStatCard />
      </ul>
    </QueryClientProvider>,
  )
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('ProjectsStatCard', () => {
  it('shows the total once loaded', async () => {
    vi.spyOn(projectsApi, 'fetchProjects').mockResolvedValue({
      data: [],
      page: 1,
      pageSize: 1,
      total: 3,
    })

    renderCard()

    expect(await screen.findByText('3')).toBeInTheDocument()
  })

  it('shows an error badge without crashing when the request fails', async () => {
    vi.spyOn(projectsApi, 'fetchProjects').mockRejectedValue(new Error('boom'))

    renderCard()

    expect(await screen.findByText('error')).toBeInTheDocument()
  })

  it('retries the query when the error badge is clicked', async () => {
    const user = userEvent.setup()
    const spy = vi
      .spyOn(projectsApi, 'fetchProjects')
      .mockRejectedValueOnce(new Error('boom'))
      .mockResolvedValue({ data: [], page: 1, pageSize: 1, total: 5 })

    renderCard()
    await user.click(await screen.findByRole('button', { name: /retry/i }))

    expect(await screen.findByText('5')).toBeInTheDocument()
    expect(spy).toHaveBeenCalledTimes(2)
  })
})
