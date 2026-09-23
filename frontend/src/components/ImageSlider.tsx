import { useState } from 'react'
import { AsciiFallback } from './AsciiFallback'
import type { ProjectImage } from '../lib/api/projects'

const NAV_BUTTON_CLASSES =
  'absolute top-1/2 -translate-y-1/2 rounded-full border border-[var(--border)] bg-[var(--panel)]/80 px-2.5 py-1 text-[var(--text)] backdrop-blur transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]'

// One project, many images: this is the slider shown on the project
// detail page. With no images at all, it falls back to the same ASCII
// fingerprint the card uses (scaled up), so a logo-only or
// not-yet-photographed project still looks intentional rather than broken.
export function ImageSlider({
  images,
  fallbackSeed,
}: {
  images: ProjectImage[]
  fallbackSeed: string
}) {
  const [index, setIndex] = useState(0)

  if (images.length === 0) {
    return (
      <div className="glow-border flex aspect-video items-center justify-center rounded border border-[var(--border)] bg-[var(--panel)]">
        <AsciiFallback seed={fallbackSeed} className="text-base" />
      </div>
    )
  }

  const current = images[Math.min(index, images.length - 1)]

  function go(next: number) {
    setIndex(((next % images.length) + images.length) % images.length)
  }

  return (
    <div>
      <div className="glow-border relative overflow-hidden rounded border border-[var(--border)] bg-black">
        <img
          src={current.url}
          alt={`Image ${index + 1} of ${images.length}`}
          className="aspect-video w-full object-contain"
        />
        {images.length > 1 && (
          <>
            <button
              type="button"
              onClick={() => go(index - 1)}
              aria-label="Previous image"
              className={`${NAV_BUTTON_CLASSES} left-2`}
            >
              &#8249;
            </button>
            <button
              type="button"
              onClick={() => go(index + 1)}
              aria-label="Next image"
              className={`${NAV_BUTTON_CLASSES} right-2`}
            >
              &#8250;
            </button>
          </>
        )}
      </div>
      {images.length > 1 && (
        <div className="mt-3 flex justify-center gap-1.5">
          {images.map((image, i) => (
            <button
              key={image.id}
              type="button"
              onClick={() => go(i)}
              aria-label={`Go to image ${i + 1}`}
              aria-current={i === index}
              className={`h-1.5 w-1.5 rounded-full transition-colors ${
                i === index ? 'bg-[var(--accent)]' : 'bg-[var(--border)] hover:bg-[var(--text-dim)]'
              }`}
            />
          ))}
        </div>
      )}
    </div>
  )
}
