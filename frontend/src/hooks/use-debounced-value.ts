import { useEffect, useState } from 'react'

// Returns `value`, but delayed until it's stopped changing for `delayMs`.
// Used to avoid firing a server request on every keystroke in a search box.
export function useDebouncedValue<T>(value: T, delayMs: number): T {
  const [debounced, setDebounced] = useState(value)

  useEffect(() => {
    const id = setTimeout(() => setDebounced(value), delayMs)
    return () => clearTimeout(id)
  }, [value, delayMs])

  return debounced
}
