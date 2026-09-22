import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { ClientsTable } from './ClientsTable'
import type { Client } from '../lib/mock-clients'

const CLIENTS: Client[] = [
  { id: '1', company: 'Nimbus Retail', contact: 'Dana Whitfield', email: 'dana@nimbusretail.test', status: 'active', projects: 3 },
  { id: '2', company: 'Cloud North Solutions', contact: 'Marcus Ellery', email: 'marcus@cloudnorth.test', status: 'prospect', projects: 0 },
]

describe('ClientsTable', () => {
  it('renders a row per client with its details', () => {
    render(<ClientsTable clients={CLIENTS} />)

    expect(screen.getByText('Nimbus Retail')).toBeInTheDocument()
    expect(screen.getByText('Dana Whitfield')).toBeInTheDocument()
    expect(screen.getByText('dana@nimbusretail.test')).toBeInTheDocument()
    expect(screen.getByText('3')).toBeInTheDocument()
  })

  it('shows the status as a badge', () => {
    render(<ClientsTable clients={CLIENTS} />)

    expect(screen.getByText('active')).toBeInTheDocument()
    expect(screen.getByText('prospect')).toBeInTheDocument()
  })

  it('shows a placeholder when there are no clients', () => {
    render(<ClientsTable clients={[]} />)

    expect(screen.getByText('-- no clients match --')).toBeInTheDocument()
  })
})
