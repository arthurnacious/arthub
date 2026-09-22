const NAV_BUTTON_CLASSES =
  'rounded border border-[var(--border)] px-2 py-1 text-xs text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:border-[var(--border)] disabled:hover:text-[var(--text-dim)]'

const PAGE_BUTTON_CLASSES =
  'min-w-[1.75rem] rounded border px-2 py-1 text-xs transition-colors'

type PageToken = number | 'ellipsis'

// First, last, and a window around the current page - the rest collapse
// into an ellipsis, so this stays readable even with dozens of pages.
function pageTokens(current: number, totalPages: number): PageToken[] {
  if (totalPages <= 7) {
    return Array.from({ length: totalPages }, (_, i) => i + 1)
  }

  const keep = new Set<number>([1, totalPages, current - 1, current, current + 1])
  const pages = Array.from(keep)
    .filter((page) => page >= 1 && page <= totalPages)
    .sort((a, b) => a - b)

  const tokens: PageToken[] = []
  let previous = 0
  for (const page of pages) {
    if (page - previous > 1) tokens.push('ellipsis')
    tokens.push(page)
    previous = page
  }
  return tokens
}

export function Pagination({
  page,
  pageSize,
  total,
  onPageChange,
}: {
  page: number
  pageSize: number
  total: number
  onPageChange: (page: number) => void
}) {
  const totalPages = Math.max(1, Math.ceil(total / pageSize))

  return (
    <div className="flex flex-wrap items-center justify-between gap-3 text-xs text-[var(--text-dim)]">
      <span>{total} total</span>
      <div className="flex items-center gap-1">
        <button
          type="button"
          onClick={() => onPageChange(page - 1)}
          disabled={page <= 1}
          className={NAV_BUTTON_CLASSES}
        >
          prev
        </button>
        {pageTokens(page, totalPages).map((token, i) =>
          token === 'ellipsis' ? (
            <span key={`ellipsis-${i}`} className="px-1">
              ...
            </span>
          ) : (
            <button
              key={token}
              type="button"
              onClick={() => onPageChange(token)}
              aria-current={token === page ? 'page' : undefined}
              className={`${PAGE_BUTTON_CLASSES} ${
                token === page
                  ? 'border-[var(--accent)] bg-[var(--accent-dim)] text-[var(--accent)]'
                  : 'border-[var(--border)] hover:border-[var(--accent)] hover:text-[var(--accent)]'
              }`}
            >
              {token}
            </button>
          ),
        )}
        <button
          type="button"
          onClick={() => onPageChange(page + 1)}
          disabled={page >= totalPages}
          className={NAV_BUTTON_CLASSES}
        >
          next
        </button>
      </div>
    </div>
  )
}
