// The "ARTHUB" wordmark: plain "ART" followed by "HUB" on a filled pill of
// the current accent color (PornHub-style logo treatment).
//
// `monochrome` drops the colored pill (and the blinking cursor) in favor of
// theme-independent text. Used by LoadingScreen, which can render before
// the theme store has synced --accent for the current color yet - a
// colored pill there could flash from the default green to the real
// color once it does. The real (branded) logo only ever mounts after
// that sync has already happened, so it never has this problem.
export function Logo({
  className = '',
  monochrome = false,
}: {
  className?: string
  monochrome?: boolean
}) {
  return (
    <div className={`flex items-center gap-0.5 font-bold tracking-widest ${className}`}>
      <span className="text-[var(--text)]">ART</span>
      <span
        className={
          monochrome
            ? 'text-[var(--text)]'
            : 'rounded bg-[var(--accent)] px-1.5 py-0.5 text-[var(--bg)]'
        }
      >
        HUB
      </span>
      {!monochrome && (
        <span className="cursor-blink text-[var(--accent)]">_</span>
      )}
    </div>
  )
}
