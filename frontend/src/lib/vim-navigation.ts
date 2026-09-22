const MOVE_KEYS = ['j', 'k', 'g', 'G', 'ArrowDown', 'ArrowUp'] as const

export type VimMoveKey = (typeof MOVE_KEYS)[number]

export function isVimMoveKey(key: string): key is VimMoveKey {
  return (MOVE_KEYS as readonly string[]).includes(key)
}

// Pure index transition for vim-style j/k/g/G movement over a list of
// `length` items. Clamps rather than wraps.
export function moveSelection(
  current: number,
  key: VimMoveKey,
  length: number,
): number {
  if (length <= 0) return current

  switch (key) {
    case 'j':
    case 'ArrowDown':
      return Math.min(length - 1, current + 1)
    case 'k':
    case 'ArrowUp':
      return Math.max(0, current - 1)
    case 'g':
      return 0
    case 'G':
      return length - 1
  }
}
