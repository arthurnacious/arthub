import { describe, expect, it } from 'vitest'
import { runCommand } from './commands'

describe('runCommand', () => {
  it('does nothing for blank input', () => {
    expect(runCommand('')).toEqual({ type: 'noop' })
    expect(runCommand('   ')).toEqual({ type: 'noop' })
  })

  it('ls lists every route name', () => {
    const result = runCommand('ls')

    expect(result.type).toBe('list')
    if (result.type === 'list') {
      expect(result.lines).toEqual([
        'dashboard',
        'clients',
        'projects',
        'quotes',
        'invoices',
        'activity',
      ])
    }
  })

  it('cd <route> navigates to a known route', () => {
    expect(runCommand('cd clients')).toEqual({
      type: 'navigate',
      path: '/clients',
      name: 'clients',
    })
  })

  it('cd tolerates a leading slash', () => {
    expect(runCommand('cd /invoices')).toEqual({
      type: 'navigate',
      path: '/invoices',
      name: 'invoices',
    })
  })

  it('cd .., cd / and cd ~ all go to the dashboard', () => {
    for (const target of ['..', '/', '~']) {
      expect(runCommand(`cd ${target}`)).toEqual({
        type: 'navigate',
        path: '/',
        name: 'dashboard',
      })
    }
  })

  it('cd with no argument is an error', () => {
    const result = runCommand('cd')

    expect(result.type).toBe('error')
  })

  it('cd to an unknown route is an error', () => {
    const result = runCommand('cd nope')

    expect(result).toEqual({
      type: 'error',
      message: 'no such route: nope',
    })
  })

  it('an unknown command is an error', () => {
    const result = runCommand('rm -rf /')

    expect(result).toEqual({
      type: 'error',
      message: 'unknown command: rm',
    })
  })

  it('trims surrounding whitespace and collapses extra spaces', () => {
    expect(runCommand('   cd    clients  ')).toEqual({
      type: 'navigate',
      path: '/clients',
      name: 'clients',
    })
  })
})
