import { act, render } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { useInfiniteScrollTrigger } from './use-infinite-scroll-trigger'
import { triggerIntersection } from '../test-utils/intersection-observer'

function Sentinel({ onIntersect, enabled }: { onIntersect: () => void; enabled: boolean }) {
  const ref = useInfiniteScrollTrigger(onIntersect, enabled)
  return <div ref={ref} data-testid="sentinel" />
}

describe('useInfiniteScrollTrigger', () => {
  it('calls onIntersect when the sentinel intersects', () => {
    const onIntersect = vi.fn()
    render(<Sentinel onIntersect={onIntersect} enabled />)

    act(() => triggerIntersection(true))

    expect(onIntersect).toHaveBeenCalledOnce()
  })

  it('does not call onIntersect when not intersecting', () => {
    const onIntersect = vi.fn()
    render(<Sentinel onIntersect={onIntersect} enabled />)

    act(() => triggerIntersection(false))

    expect(onIntersect).not.toHaveBeenCalled()
  })

  it('does not observe anything when disabled', () => {
    const onIntersect = vi.fn()
    render(<Sentinel onIntersect={onIntersect} enabled={false} />)

    act(() => triggerIntersection(true))

    expect(onIntersect).not.toHaveBeenCalled()
  })
})
