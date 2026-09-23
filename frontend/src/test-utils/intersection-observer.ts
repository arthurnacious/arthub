// jsdom doesn't implement IntersectionObserver at all, so anything that
// constructs one (useInfiniteScrollTrigger) would throw under test without
// this. Installed globally in test-setup.ts; tests that need to simulate
// a sentinel scrolling into view call triggerIntersection().
export class FakeIntersectionObserver implements IntersectionObserver {
  static instances: FakeIntersectionObserver[] = []

  readonly root = null
  readonly rootMargin = ''
  readonly thresholds: ReadonlyArray<number> = []
  observedElements: Element[] = []

  constructor(private callback: IntersectionObserverCallback) {
    FakeIntersectionObserver.instances.push(this)
  }

  observe(element: Element): void {
    this.observedElements.push(element)
  }

  unobserve(element: Element): void {
    this.observedElements = this.observedElements.filter((el) => el !== element)
  }

  disconnect(): void {
    this.observedElements = []
  }

  takeRecords(): IntersectionObserverEntry[] {
    return []
  }

  fire(isIntersecting: boolean): void {
    const entries = this.observedElements.map(
      (target) => ({ isIntersecting, target }) as IntersectionObserverEntry,
    )
    this.callback(entries, this)
  }
}

export function installFakeIntersectionObserver(): void {
  FakeIntersectionObserver.instances = []
  ;(globalThis as unknown as { IntersectionObserver: unknown }).IntersectionObserver =
    FakeIntersectionObserver
}

/** Simulates every currently-observed sentinel scrolling into (or out of) view. */
export function triggerIntersection(isIntersecting = true): void {
  for (const instance of FakeIntersectionObserver.instances) {
    instance.fire(isIntersecting)
  }
}
