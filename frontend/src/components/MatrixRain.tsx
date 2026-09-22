import { useEffect, useRef } from 'react'
import { useStore } from '@tanstack/react-store'
import { themeStore } from '../stores/theme-store'
import type { ThemeColor } from '../stores/theme-store'

const ACCENT_HEX: Record<ThemeColor, string> = {
  green: '#4ade80',
  purple: '#c084fc',
  orange: '#fb923c',
  red: '#f87171',
}

const GLYPHS =
  'アイウエオカキクケコサシスセソタチツテト0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ$+-*/=%<>[]{}'
const FONT_SIZE = 16
// Redrawing every rAF tick (~60fps) is wasted work for an effect that
// reads fine at a much lower cadence - throttling to this interval keeps
// it cheap and gives the classic "digital rain" step-y feel.
const FRAME_INTERVAL_MS = 60

// Canvas-based Matrix-style falling glyph rain, tinted to the current
// theme accent. Canvas (redrawn at a throttled interval, not every frame)
// is dramatically cheaper here than animating hundreds of DOM nodes would
// be. Reduced-motion users get a static single frame instead.
export function MatrixRain() {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const color = useStore(themeStore.store, (state) => state)
  const colorRef = useRef(color)
  colorRef.current = color

  useEffect(() => {
    const canvas = canvasRef.current
    const ctx = canvas?.getContext('2d')
    if (!canvas || !ctx) return

    const reduceMotion = window.matchMedia(
      '(prefers-reduced-motion: reduce)',
    ).matches

    let columns = 0
    let drops: number[] = []

    function resize() {
      canvas!.width = window.innerWidth
      canvas!.height = window.innerHeight
      columns = Math.ceil(canvas!.width / FONT_SIZE)
      drops = Array.from({ length: columns }, () =>
        Math.floor((Math.random() * canvas!.height) / -FONT_SIZE),
      )
    }

    function drawFrame() {
      ctx!.fillStyle = 'rgba(10, 10, 10, 0.08)'
      ctx!.fillRect(0, 0, canvas!.width, canvas!.height)

      ctx!.font = `${FONT_SIZE}px var(--font-mono, monospace)`
      ctx!.fillStyle = ACCENT_HEX[colorRef.current]

      for (let i = 0; i < drops.length; i++) {
        const glyph = GLYPHS[Math.floor(Math.random() * GLYPHS.length)]
        ctx!.fillText(glyph, i * FONT_SIZE, drops[i] * FONT_SIZE)

        if (drops[i] * FONT_SIZE > canvas!.height && Math.random() > 0.975) {
          drops[i] = 0
        }
        drops[i]++
      }
    }

    resize()
    window.addEventListener('resize', resize)

    if (reduceMotion) {
      drawFrame()
      return () => window.removeEventListener('resize', resize)
    }

    let rafId: number
    let lastFrame = 0

    function tick(timestamp: number) {
      rafId = requestAnimationFrame(tick)
      if (timestamp - lastFrame < FRAME_INTERVAL_MS) return
      lastFrame = timestamp
      drawFrame()
    }

    rafId = requestAnimationFrame(tick)

    return () => {
      cancelAnimationFrame(rafId)
      window.removeEventListener('resize', resize)
    }
  }, [])

  return <canvas ref={canvasRef} className="matrix-rain" aria-hidden="true" />
}
