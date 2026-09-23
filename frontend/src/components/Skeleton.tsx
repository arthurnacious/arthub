// A single pulsing placeholder bar/block - the building block every
// page-specific loading skeleton (table rows, cards, detail-page fields)
// is composed from. Purely decorative, so it's hidden from assistive
// tech; the page's own "loading" state should be announced separately
// (e.g. via aria-busy on the containing region) if that matters.
export function Skeleton({ className = '' }: { className?: string }) {
  return <div aria-hidden="true" className={`animate-pulse rounded bg-[var(--border)] ${className}`} />
}
