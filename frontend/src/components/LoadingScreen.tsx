import { Logo } from './Logo'

// Shown until the client has mounted and synced the theme color from
// localStorage (see __root.tsx) - avoids a flash of the wrong accent color
// (or wrong data) on load. Deliberately theme-independent (monochrome
// logo) since it's what's on screen *during* that sync.
export function LoadingScreen() {
  return (
    <div className="fixed inset-0 z-[100] flex flex-col items-center justify-center gap-3 bg-[var(--bg)]">
      <Logo monochrome className="text-2xl" />
      <p className="text-xs text-[var(--text-dim)]">booting...</p>
    </div>
  )
}
