import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'
import { ImageSlider } from './ImageSlider'
import type { ProjectImage } from '../lib/api/projects'

const IMAGES: ProjectImage[] = [
  { id: '1', url: 'http://x/1.png', sortOrder: 0 },
  { id: '2', url: 'http://x/2.png', sortOrder: 1 },
  { id: '3', url: 'http://x/3.png', sortOrder: 2 },
]

describe('ImageSlider', () => {
  it('shows the ASCII fallback and no nav controls when there are no images', () => {
    render(<ImageSlider images={[]} fallbackSeed="project-1" />)

    expect(document.querySelector('pre')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /next image/i })).not.toBeInTheDocument()
  })

  it('shows the first image and no nav controls with a single image', () => {
    render(<ImageSlider images={[IMAGES[0]]} fallbackSeed="project-1" />)

    expect(screen.getByAltText('Image 1 of 1')).toHaveAttribute('src', 'http://x/1.png')
    expect(screen.queryByRole('button', { name: /next image/i })).not.toBeInTheDocument()
  })

  it('advances to the next image on click and wraps around', async () => {
    const user = userEvent.setup()
    render(<ImageSlider images={IMAGES} fallbackSeed="project-1" />)

    expect(screen.getByAltText('Image 1 of 3')).toHaveAttribute('src', 'http://x/1.png')

    await user.click(screen.getByRole('button', { name: /next image/i }))
    expect(screen.getByAltText('Image 2 of 3')).toHaveAttribute('src', 'http://x/2.png')

    await user.click(screen.getByRole('button', { name: /next image/i }))
    await user.click(screen.getByRole('button', { name: /next image/i }))
    expect(screen.getByAltText('Image 1 of 3')).toHaveAttribute('src', 'http://x/1.png')
  })

  it('goes back to the previous image and wraps around at the start', async () => {
    const user = userEvent.setup()
    render(<ImageSlider images={IMAGES} fallbackSeed="project-1" />)

    await user.click(screen.getByRole('button', { name: /previous image/i }))
    expect(screen.getByAltText('Image 3 of 3')).toHaveAttribute('src', 'http://x/3.png')
  })

  it('jumps directly to an image via its dot indicator', async () => {
    const user = userEvent.setup()
    render(<ImageSlider images={IMAGES} fallbackSeed="project-1" />)

    await user.click(screen.getByRole('button', { name: 'Go to image 3' }))
    expect(screen.getByAltText('Image 3 of 3')).toHaveAttribute('src', 'http://x/3.png')
  })
})
