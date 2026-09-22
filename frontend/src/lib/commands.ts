import { ROUTES } from './routes'

export type CommandResult =
  | { type: 'noop' }
  | { type: 'list'; lines: string[] }
  | { type: 'navigate'; path: string; name: string }
  | { type: 'error'; message: string }

const DASHBOARD_ALIASES = new Set(['..', '/', '~'])

function normalizeTarget(target: string): string {
  if (DASHBOARD_ALIASES.has(target)) return 'dashboard'
  return target.replace(/^\//, '')
}

// Parses a `:`-command-bar input into a result the CommandBar can act on.
// Supported commands, vim/shell inspired:
//   ls          list every route name
//   cd <route>  navigate to a route (also: cd .., cd /, cd ~ for dashboard)
export function runCommand(input: string): CommandResult {
  const trimmed = input.trim()
  if (trimmed === '') return { type: 'noop' }

  const [cmd, ...args] = trimmed.split(/\s+/)

  switch (cmd) {
    case 'ls':
      return { type: 'list', lines: ROUTES.map((route) => route.name) }

    case 'cd': {
      const target = args[0]
      if (!target) return { type: 'error', message: 'usage: cd <route>' }

      const name = normalizeTarget(target)
      const route = ROUTES.find((candidate) => candidate.name === name)
      if (!route) return { type: 'error', message: `no such route: ${target}` }

      return { type: 'navigate', path: route.path, name: route.name }
    }

    default:
      return { type: 'error', message: `unknown command: ${cmd}` }
  }
}
