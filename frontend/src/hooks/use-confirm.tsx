import { useCallback, useRef, useState } from 'react'
import { ConfirmDialog } from '../components/ConfirmDialog'
import type { ConfirmDialogProps } from '../components/ConfirmDialog'

export type ConfirmOptions = Omit<ConfirmDialogProps, 'onConfirm' | 'onCancel'>

/**
 * Promise-based confirmation dialog. Call `confirm({ message })` and await
 * it - resolves `true`/`false` depending on what the user picked. Render
 * the returned `dialog` node somewhere in the tree (it's `null` until a
 * confirm is in flight).
 *
 *   const { confirm, dialog } = useConfirm()
 *   async function handleDelete() {
 *     if (!(await confirm({ message: 'Delete this?', tone: 'danger' }))) return
 *     await deleteThing()
 *   }
 *   return <>...{dialog}</>
 */
export function useConfirm() {
  const [options, setOptions] = useState<ConfirmOptions | null>(null)
  const resolveRef = useRef<((value: boolean) => void) | null>(null)

  const confirm = useCallback((opts: ConfirmOptions) => {
    return new Promise<boolean>((resolve) => {
      resolveRef.current = resolve
      setOptions(opts)
    })
  }, [])

  function settle(value: boolean) {
    resolveRef.current?.(value)
    resolveRef.current = null
    setOptions(null)
  }

  const dialog = options ? (
    <ConfirmDialog
      {...options}
      onConfirm={() => settle(true)}
      onCancel={() => settle(false)}
    />
  ) : null

  return { confirm, dialog }
}
