import { useEffect, useRef } from 'react'

// Attach the returned ref to a sentinel element at the end of a list;
// onIntersect fires once whenever it scrolls into view while `enabled` is
// true (e.g. hasNextPage && !isFetchingNextPage), driving infinite scroll
// without a "load more" button.
export function useInfiniteScrollTrigger(onIntersect: () => void, enabled: boolean) {
  const targetRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!enabled) return

    const target = targetRef.current
    if (!target) return

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting) onIntersect()
      },
      { rootMargin: '400px' },
    )

    observer.observe(target)
    return () => observer.disconnect()
  }, [onIntersect, enabled])

  return targetRef
}
