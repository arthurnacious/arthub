import { useMemo } from 'react'
import { generateAsciiArt } from '../lib/ascii-art'

// A generated ASCII "fingerprint" shown wherever a project has no uploaded
// image - deterministic per seed (use the project id), so it's stable
// across renders but visually distinct project to project.
export function AsciiFallback({
  seed,
  className = '',
}: {
  seed: string
  className?: string
}) {
  const art = useMemo(() => generateAsciiArt(seed), [seed])

  return (
    <pre
      aria-hidden="true"
      className={`glow-text pointer-events-none select-none whitespace-pre text-center font-mono leading-none text-[var(--accent)] opacity-70 ${className}`}
    >
      {art}
    </pre>
  )
}
