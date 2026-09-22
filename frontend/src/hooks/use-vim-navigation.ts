import { useEffect, useState } from 'react'
import { useLocation, useNavigate, useRouter } from '@tanstack/react-router'
import { ROUTES } from '../lib/routes'
import { isVimMoveKey, moveSelection } from '../lib/vim-navigation'

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  return (
    target.tagName === 'INPUT' ||
    target.tagName === 'TEXTAREA' ||
    target.isContentEditable
  )
}

function indexForPath(pathname: string): number {
  const index = ROUTES.findIndex((route) => route.path === pathname)
  return index === -1 ? 0 : index
}

/**
 * Vim-inspired keyboard navigation across the sidebar routes:
 *   j / ArrowDown   select the next route
 *   k / ArrowUp     select the previous route
 *   g               select the first route
 *   G               select the last route
 *   l / Enter       go to the selected route
 *   h               go back in history
 *
 * Automatically ignored while focus is in a text input (e.g. the
 * CommandBar), so it never steals keystrokes from typing.
 */
export function useVimNavigation() {
  const location = useLocation()
  const navigate = useNavigate()
  const router = useRouter()
  const [selectedIndex, setSelectedIndex] = useState(() =>
    indexForPath(location.pathname),
  )

  useEffect(() => {
    setSelectedIndex(indexForPath(location.pathname))
  }, [location.pathname])

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (isTypingTarget(event.target)) return
      if (event.metaKey || event.ctrlKey || event.altKey) return

      if (isVimMoveKey(event.key)) {
        const key = event.key
        event.preventDefault()
        setSelectedIndex((current) => moveSelection(current, key, ROUTES.length))
        return
      }

      if (event.key === 'h') {
        event.preventDefault()
        router.history.back()
        return
      }

      if (event.key === 'l' || event.key === 'Enter') {
        event.preventDefault()
        setSelectedIndex((current) => {
          navigate({ to: ROUTES[current].path })
          return current
        })
      }
    }

    document.addEventListener('keydown', onKeyDown)
    return () => document.removeEventListener('keydown', onKeyDown)
  }, [navigate, router])

  return { selectedIndex }
}
