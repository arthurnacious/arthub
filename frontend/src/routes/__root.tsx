import {
  HeadContent,
  Scripts,
  createRootRouteWithContext,
} from '@tanstack/react-router'
import { TanStackRouterDevtoolsPanel } from '@tanstack/react-router-devtools'
import { TanStackDevtools } from '@tanstack/react-devtools'

import TanStackQueryDevtools from '../integrations/tanstack-query/devtools'
import { AppShell } from '../components/AppShell'
import { ScanlineOverlay } from '../components/ScanlineOverlay'
import { MatrixRain } from '../components/MatrixRain'
import { StatusBar } from '../components/StatusBar'
import { CommandBar } from '../components/CommandBar'
import { LoadingScreen } from '../components/LoadingScreen'
import { useSyncThemeColor } from '../hooks/use-sync-theme-color'
import { useAppReady } from '../hooks/use-app-ready'

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
        title: import.meta.env.VITE_SITE_NAME ?? 'Arthub',
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
  // Declaration order matters: useSyncThemeColor's effect must register
  // (and therefore run) before useAppReady's, so --accent is already
  // correct by the time `ready` flips true and reveals the real app.
  useSyncThemeColor()
  const ready = useAppReady()

  return (
    <html lang="en">
      <head>
        <HeadContent />
      </head>
      <body className="flex flex-col overflow-hidden">
        <MatrixRain />
        <ScanlineOverlay />
        {ready ? (
          <>
            <AppShell>{children}</AppShell>
            <StatusBar />
            <CommandBar />
          </>
        ) : (
          <LoadingScreen />
        )}
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
