// Purely decorative CRT effect (static scanlines + a slow sweeping band),
// styled in styles.css. aria-hidden and pointer-events-none so it never
// affects layout, a11y tree or interaction.
export function ScanlineOverlay() {
  return <div className="scanline-overlay" aria-hidden="true" />
}
