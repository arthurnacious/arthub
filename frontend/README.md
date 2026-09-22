# Arthub frontend

The dashboard UI for [Arthub](../README.md), a freelancer CRM. Built with
[TanStack Start](https://tanstack.com/start) (React), TanStack
Router/Query/Store, and Tailwind CSS v4, styled as a dark, retro terminal:
a bordered panel-grid dashboard, vim-style keyboard navigation, and a
`:`-prefixed command bar.

Currently runs entirely on mock data (see the `MOCK_*` constants in
`src/routes/index.tsx` and friends) - there's no login flow yet, so nothing
talks to the real backend. See the root [README](../README.md#status) for
where things stand.

## Setup

```bash
bun install
cp .env.example .env   # fill in VITE_API_URL if the backend isn't on the default port
bun --bun run dev      # serves on localhost:3000
```

The backend ([`../backend`](../backend)) needs to be running separately -
see its own README - for anything beyond the mock-data dashboard.

## Scripts

```bash
bun --bun run dev      # dev server
bun --bun run build    # production build
bun run test           # Vitest
```

## Testing

TDD, same as the backend - write the test first. Uses
[Vitest](https://vitest.dev/) + [Testing Library](https://testing-library.com/).
Component tests that need router context (anything using `Link`,
`useNavigate`, `useLocation`, etc.) build a small in-memory `RouterProvider`
per test rather than mocking the router - see `CommandBar.test.tsx` or
`use-vim-navigation.test.tsx` for the pattern.

## Notable pieces

- `src/components/AppShell.tsx`, `TopBar.tsx`, `Panel.tsx`, `Mnemonic.tsx` -
  the terminal-dashboard shell: sidebar nav, top bar, bordered panel
  chrome, and the "first-letter-highlighted" label style used throughout.
- `src/hooks/use-vim-navigation.ts` + `src/lib/vim-navigation.ts` - the
  `j`/`k`/`g`/`G`/`h`/`l` keyboard navigation over the sidebar routes.
- `src/components/CommandBar.tsx` + `src/lib/commands.ts` - the `:`
  command bar (`ls`, `cd <route>`).
- `src/stores/theme-store.ts` - the accent-color theme (green/purple/
  orange/red), backed by TanStack Store and `localStorage`.
- `src/components/MatrixRain.tsx` - the canvas background effect.

## Routing

File-based, via TanStack Router - add a file under `src/routes/` and the
route tree regenerates automatically. The shared shell (root route) lives
in `src/routes/__root.tsx`.
