import {
  HeadContent,
  Scripts,
  createRootRouteWithContext,
} from '@tanstack/react-router'
import { TanStackRouterDevtoolsPanel } from '@tanstack/react-router-devtools'
import { TanStackDevtools } from '@tanstack/react-devtools'

import TanStackQueryDevtools from '../integrations/tanstack-query/devtools'
import { ThemeToggle } from '../components/ThemeToggle'
import { AppShell } from '../components/AppShell'
import { ScanlineOverlay } from '../components/ScanlineOverlay'
import { CommandBar } from '../components/CommandBar'
import { useSyncThemeColor } from '../hooks/use-sync-theme-color'

import appCss from '../styles.css?url'

import type { QueryClient } from '@tanstack/react-query'

interface MyRouterContext {
  queryClient: QueryClient
}

export const Route = createRootRouteWithContext<MyRouterContext>()({
  head: () => ({
    meta: [
      {
        charSet: 'utf-8',
      },
      {
        name: 'viewport',
        content: 'width=device-width, initial-scale=1',
      },
      {
        title: 'TanStack Start Starter',
      },
    ],
    links: [
      {
        rel: 'stylesheet',
        href: appCss,
      },
    ],
  }),
  shellComponent: RootDocument,
})

function RootDocument({ children }: { children: React.ReactNode }) {
  useSyncThemeColor()

  return (
    <html lang="en">
      <head>
        <HeadContent />
      </head>
      <body>
        <ScanlineOverlay />
        <div className="fixed right-4 top-4 z-50">
          <ThemeToggle />
        </div>
        <AppShell>{children}</AppShell>
        <CommandBar />
        <TanStackDevtools
          config={{
            position: 'bottom-right',
          }}
          plugins={[
            {
              name: 'Tanstack Router',
              render: <TanStackRouterDevtoolsPanel />,
            },
            TanStackQueryDevtools,
          ]}
        />
        <Scripts />
      </body>
    </html>
  )
}
