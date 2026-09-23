import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { ProjectCard } from './ProjectCard'
import type { Project } from '../lib/api/projects'

const PROJECT: Project = {
  id: '1',
  clientId: 'c1',
  clientName: 'Nimbus Retail',
  name: 'Marketing Site',
  description: '',
  category: 'web',
  url: 'https://example.com',
  status: 'active',
  coverImage: null,
}

describe('ProjectCard', () => {
  it('shows an ASCII fallback when there is no cover image', () => {
    render(<ProjectCard project={PROJECT} onClick={() => {}} />)

    expect(document.querySelector('pre')).toBeInTheDocument()
    expect(screen.queryByRole('img')).not.toBeInTheDocument()
  })

  it('shows the cover image instead of the fallback when one exists', () => {
    render(<ProjectCard project={{ ...PROJECT, coverImage: 'http://x/cover.png' }} onClick={() => {}} />)

    expect(document.querySelector('pre')).not.toBeInTheDocument()
    expect(screen.getByRole('img')).toHaveAttribute('src', 'http://x/cover.png')
  })

  it('shows the project name, client, category and status', () => {
    render(<ProjectCard project={PROJECT} onClick={() => {}} />)

    expect(screen.getByText('Marketing Site')).toBeInTheDocument()
    expect(screen.getByText('Nimbus Retail')).toBeInTheDocument()
    expect(screen.getByText('web')).toBeInTheDocument()
    expect(screen.getByText('active')).toBeInTheDocument()
  })

  it('calls onClick when clicked', async () => {
    const user = userEvent.setup()
    const onClick = vi.fn()
    render(<ProjectCard project={PROJECT} onClick={onClick} />)

    await user.click(screen.getByRole('button'))

    expect(onClick).toHaveBeenCalledOnce()
  })
})
