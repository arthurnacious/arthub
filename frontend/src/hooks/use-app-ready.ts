import { useEffect, useState } from 'react'

/**
 * True only after the client has mounted. False during SSR and the very
 * first client render, so callers (see src/routes/__root.tsx) can hold off
 * on rendering theme/localStorage-backed UI until after that first effect
 * pass - avoiding a flash of the wrong theme color once real values load.
 */
export function useAppReady(): boolean {
  const [ready, setReady] = useState(false)

  useEffect(() => {
    setReady(true)
  }, [])

  return ready
}
