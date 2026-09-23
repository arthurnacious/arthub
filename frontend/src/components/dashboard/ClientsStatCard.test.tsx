import { render, screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { ClientsStatCard } from './ClientsStatCard'
import * as clientsApi from '../../lib/api/clients'

function renderCard() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <ul>
        <ClientsStatCard />
      </ul>
    </QueryClientProvider>,
  )
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('ClientsStatCard', () => {
  it('shows the total once loaded', async () => {
    vi.spyOn(clientsApi, 'fetchClients').mockResolvedValue({
      data: [],
      page: 1,
      pageSize: 1,
      total: 7,
    })

    renderCard()

    expect(await screen.findByText('7')).toBeInTheDocument()
  })

  it('shows an error badge without crashing when the request fails', async () => {
    vi.spyOn(clientsApi, 'fetchClients').mockRejectedValue(new Error('boom'))

    renderCard()

    expect(await screen.findByText('error')).toBeInTheDocument()
  })
})
