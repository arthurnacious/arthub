import { useMemo } from 'react'

interface Blip {
  id: number
  top: string
  left: string
  size: number
  delay: string
  duration: string
}

function randomBlips(count: number): Blip[] {
  return Array.from({ length: count }, (_, id) => ({
    id,
    top: `${Math.round(Math.random() * 100)}%`,
    left: `${Math.round(Math.random() * 100)}%`,
    size: 2 + Math.round(Math.random() * 3),
    delay: `${(Math.random() * 6).toFixed(2)}s`,
    duration: `${(3 + Math.random() * 4).toFixed(2)}s`,
  }))
}

// Decorative radar-scope background: a rotating sweep, static range rings
// and a handful of "blips" whose positions/timing are randomized once per
// mount, then animated purely in CSS (transform/opacity only) - no
// per-frame JS, so it never costs real frames.
export function RadarBackground() {
  const blips = useMemo(() => randomBlips(9), [])

  return (
    <div className="radar-bg" aria-hidden="true">
      <div className="radar-rings" />
      <div className="radar-sweep" />
      {blips.map((blip) => (
        <span
          key={blip.id}
          className="radar-blip"
          style={{
            top: blip.top,
            left: blip.left,
            width: blip.size,
            height: blip.size,
            animationDelay: blip.delay,
            animationDuration: blip.duration,
          }}
        />
      ))}
    </div>
  )
}
