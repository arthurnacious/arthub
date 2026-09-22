// Single source of truth for navigable app routes: used by the sidebar nav,
// the vim-style keyboard navigation and the `:ls`/`:cd` command bar.
export interface RouteEntry {
  name: string
  path: string
}

export const ROUTES: RouteEntry[] = [
  { name: 'dashboard', path: '/' },
  { name: 'clients', path: '/clients' },
  { name: 'projects', path: '/projects' },
  { name: 'quotes', path: '/quotes' },
  { name: 'invoices', path: '/invoices' },
  { name: 'activity', path: '/activity' },
]
