import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ProjectForm } from './ProjectForm'
import type { ProjectFormValues } from './ProjectForm'
import * as clientsApi from '../lib/api/clients'
import * as projectsApi from '../lib/api/projects'

const EMPTY: ProjectFormValues = {
  clientId: '',
  name: '',
  description: '',
  category: '',
  url: '',
  status: 'active',
}

function renderForm(props: Partial<Parameters<typeof ProjectForm>[0]> = {}) {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const onChange = vi.fn()
  const onSubmit = vi.fn()
  const onCancel = vi.fn()

  const result = render(
    <QueryClientProvider client={queryClient}>
      <ProjectForm
        values={EMPTY}
        onChange={onChange}
        onSubmit={onSubmit}
        onCancel={onCancel}
        submitLabel="create"
        submitting={false}
        {...props}
      />
    </QueryClientProvider>,
  )

  return { ...result, onChange, onSubmit, onCancel }
}

beforeEach(() => {
  vi.spyOn(clientsApi, 'fetchClients').mockResolvedValue({
    data: [
      { id: 'c1', company: 'Nimbus Retail', contact: 'Dana', email: 'dana@x.test', status: 'active', projects: 0 },
    ],
    page: 1,
    pageSize: 200,
    total: 1,
  })
  vi.spyOn(projectsApi, 'fetchProjectCategories').mockResolvedValue(['branding', 'web'])
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('ProjectForm', () => {
  it('lists fetched clients in the client select', async () => {
    renderForm()

    expect(await screen.findByRole('option', { name: 'Nimbus Retail' })).toBeInTheDocument()
  })

  it('calls onChange with the updated name as the user types', async () => {
    const user = userEvent.setup()
    const { onChange } = renderForm()

    await user.type(screen.getByLabelText('name'), 'X')

    expect(onChange).toHaveBeenCalledWith({ ...EMPTY, name: 'X' })
  })

  it('disables submit until a client is selected', () => {
    renderForm({ values: EMPTY });

    expect(screen.getByRole('button', { name: 'create' })).toBeDisabled()
  })

  it('enables submit once a client is selected', () => {
    renderForm({ values: { ...EMPTY, clientId: 'c1', name: 'Site' } })

    expect(screen.getByRole('button', { name: 'create' })).toBeEnabled()
  })

  it('calls onSubmit when the form is submitted', async () => {
    const user = userEvent.setup()
    const { onSubmit } = renderForm({ values: { ...EMPTY, clientId: 'c1', name: 'Site' } })

    await user.click(screen.getByRole('button', { name: 'create' }))

    expect(onSubmit).toHaveBeenCalledOnce()
  })

  it('calls onCancel when cancel is clicked', async () => {
    const user = userEvent.setup()
    const { onCancel } = renderForm()

    await user.click(screen.getByRole('button', { name: 'cancel' }))

    expect(onCancel).toHaveBeenCalledOnce()
  })
})
