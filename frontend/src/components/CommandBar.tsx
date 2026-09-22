import { useEffect, useRef, useState } from 'react'
import { useNavigate } from '@tanstack/react-router'
import { runCommand } from '../lib/commands'

interface OutputLine {
  id: number
  text: string
  tone: 'output' | 'error'
}

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  return (
    target.tagName === 'INPUT' ||
    target.tagName === 'TEXTAREA' ||
    target.isContentEditable
  )
}

let nextLineId = 0

// The `:` command bar: vim-style, opened with the ':' key anywhere outside
// a text input. Supports `ls` (list routes) and `cd <route>` (navigate),
// parsed by src/lib/commands.ts.
export function CommandBar() {
  const [open, setOpen] = useState(false)
  const [value, setValue] = useState('')
  const [lines, setLines] = useState<OutputLine[]>([])
  const inputRef = useRef<HTMLInputElement>(null)
  const navigate = useNavigate()

  useEffect(() => {
    if (open) return

    function onKeyDown(event: KeyboardEvent) {
      if (isTypingTarget(event.target)) return
      if (event.key !== ':') return

      event.preventDefault()
      setOpen(true)
    }

    document.addEventListener('keydown', onKeyDown)
    return () => document.removeEventListener('keydown', onKeyDown)
  }, [open])

  useEffect(() => {
    if (open) inputRef.current?.focus()
  }, [open])

  function close() {
    setOpen(false)
    setValue('')
    setLines([])
  }

  function submit() {
    const result = runCommand(value)
    setValue('')

    if (result.type === 'list') {
      setLines((prev) => [
        ...prev,
        ...result.lines.map((text) => ({
          id: nextLineId++,
          text,
          tone: 'output' as const,
        })),
      ])
    } else if (result.type === 'navigate') {
      navigate({ to: result.path })
      close()
    } else if (result.type === 'error') {
      setLines((prev) => [
        ...prev,
        { id: nextLineId++, text: result.message, tone: 'error' as const },
      ])
    }
  }

  return (
    <>
      {/* Always rendered so the mount/keydown-listener effect above can be
          observed in tests before the ':' shortcut is exercised. */}
      <span data-testid="command-bar-mounted" className="hidden" aria-hidden="true" />
      {open && (
        <div className="glow-border fixed inset-x-0 bottom-0 z-50 border-t border-[var(--border)] bg-[var(--panel)] p-3 text-sm">
          {lines.length > 0 && (
            <ul className="mb-2 max-h-40 space-y-0.5 overflow-y-auto">
              {lines.map((line) => (
                <li
                  key={line.id}
                  className={
                    line.tone === 'error' ? 'text-red-400' : 'text-[var(--text-dim)]'
                  }
                >
                  {line.text}
                </li>
              ))}
            </ul>
          )}
          <div className="flex items-center gap-2">
            <span className="text-[var(--accent)]">:</span>
            <input
              ref={inputRef}
              value={value}
              onChange={(event) => setValue(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') {
                  event.preventDefault()
                  submit()
                } else if (event.key === 'Escape') {
                  event.preventDefault()
                  close()
                }
              }}
              className="flex-1 bg-transparent text-[var(--text)] outline-none"
              placeholder="ls, cd <route>"
              aria-label="Command"
            />
          </div>
        </div>
      )}
    </>
  )
}
