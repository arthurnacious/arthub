import { useMemo } from 'react'

// The sweep's full rotation period. Shared (inline, not via CSS) with every
// blip's animation-duration/delay below, so a blip only lights up the exact
// instant the rotating line crosses its angle - not on its own clock.
const SWEEP_DURATION_S = 9

interface Blip {
  id: number
  top: string
  left: string
  size: number
  delay: string
}

// Angle (degrees, clockwise from straight up) of a point at (leftPct,
// topPct) as seen from the center of the panel - matches the compass
// convention conic-gradient's "from 0deg" sweep uses.
function angleFromCenter(leftPct: number, topPct: number): number {
  const dx = leftPct - 50
  const dy = topPct - 50
  const degrees = (Math.atan2(dx, -dy) * 180) / Math.PI
  return (degrees + 360) % 360
}

function randomBlips(count: number): Blip[] {
  return Array.from({ length: count }, (_, id) => {
    const leftPct = Math.round(Math.random() * 100)
    const topPct = Math.round(Math.random() * 100)
    const angle = angleFromCenter(leftPct, topPct)
    const delay = (angle / 360) * SWEEP_DURATION_S

    return {
      id,
      top: `${topPct}%`,
      left: `${leftPct}%`,
      size: 2 + Math.round(Math.random() * 3),
      delay: `${delay.toFixed(3)}s`,
    }
  })
}

// Decorative radar-scope background: a rotating sweep, static range rings
// and a handful of "blips" that only appear once the sweep line passes
// over them, then fade slowly until its next pass. Positions are
// randomized once per mount; every animation here is transform/opacity
// only (compositor-only), so it costs nothing per frame.
export function RadarBackground() {
  const blips = useMemo(() => randomBlips(9), [])

  return (
    <div className="radar-bg" aria-hidden="true">
      <div className="radar-rings" />
      <div
        className="radar-sweep"
        style={{ animationDuration: `${SWEEP_DURATION_S}s` }}
      />
      {blips.map((blip) => (
        <span
          key={blip.id}
          className="radar-blip"
          style={{
            top: blip.top,
            left: blip.left,
            width: blip.size,
            height: blip.size,
            animationDuration: `${SWEEP_DURATION_S}s`,
            animationDelay: blip.delay,
          }}
        />
      ))}
    </div>
  )
}
