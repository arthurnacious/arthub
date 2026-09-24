# Arthub app

A Flutter client for [Arthub](../README.md)'s backend API, styled to match
the [frontend](../frontend)'s dark, retro-terminal look (JetBrains Mono,
bordered "panel" chrome, the same accent green and color tokens - see
`lib/core/theme/`).

## Status: scaffold

This is a starting point, not a feature-complete app. What's wired up:

- **Auth** - real login against `POST /auth/login`, session token in
  secure storage, restored (and re-verified via `GET /auth/me`) on app
  start, auto-logout on a 401 from any request.
- **Navigation** - a bottom `NavigationBar` (dashboard/clients/projects/
  quotes/invoices - Material caps a bottom nav at 5 destinations, so
  activity lives behind a history icon in the AppBar instead, pushed
  rather than a tab), with an `AuthBloc`-driven redirect guard
  (go_router) so every screen requires a session, same as every route on
  the web app. This is deliberately *not* the web frontend's sidebar:
  same look and feel (theme, panel chrome), but this is a real Android
  app, and bottom nav is the idiomatic pattern for that, not a sidebar
  meant for a wide desktop viewport.
- **Clients** - the one fully-built-out CRUD feature. A searched,
  infinite-scroll list (`GET /clients`) that pushes a real detail screen
  (`GET /clients/{id}`) with in-place edit (`PATCH`) and delete
  (`DELETE`, behind a confirm dialog), plus a "new client" screen
  (`POST`). One bloc per screen instance (`ClientsBloc` is app-wide;
  `ClientDetailBloc`/`ClientCreateBloc` are created per push via
  `Navigator`, not app-wide) - see `lib/features/clients/bloc/`.
- **Dashboard** - three independent panels (`status`, `invoice_status`,
  `recent_activity`), each its own bloc, matching the web frontend's
  "each panel fetches its own data independently" dashboard philosophy
  (frontend/src/routes/index.tsx) - one panel's error/loading state
  never blocks another's. No activity heatmap or revenue chart here
  (the web dashboard's `activity_graph`/`revenue` panels) - those are
  rendered as plain numbers instead of charts, a reasonable follow-up
  rather than something this pass attempted. `lib/features/projects/`,
  `lib/features/invoices/`, `lib/features/activity/` exist only as
  thin, dashboard-scoped repositories (a project count, an invoice list
  for aggregation, a recent-activity list) - not full features yet.

Everything but clients and the dashboard (projects/quotes/invoices/
activity as their own screens) is a placeholder. Follow
`lib/features/clients/`'s `data/` → `bloc/` → `presentation/` split to
build one out for real - see the corresponding controller under
`../backend/src/Modules/` for the API shape, and the equivalent page
under `../frontend/src/routes/` for what it should end up looking like.

## Setup

Needs the Flutter SDK - this repo pins the version via `mise` (see
`../mise.toml`):

```bash
mise install    # installs the pinned Flutter version, from the repo root
cd app
flutter pub get
```

The backend ([`../backend`](../backend)) needs to be running separately.
By default this app points at `http://192.168.0.170:9501` (the same LAN IP
the web frontend's `.env` uses - see `../backend/.env`'s
`CORS_ALLOWED_ORIGIN` comment for why a LAN IP, not `localhost`, is the
default: a physical phone on the same network can't reach the dev
machine's `localhost`). Override it per-run if that's not your setup:

```bash
flutter run --dart-define=API_BASE_URL=http://10.0.2.2:9501   # Android emulator
flutter run --dart-define=API_BASE_URL=http://localhost:9501  # iOS simulator
```

## Testing

```bash
flutter test
```

Model/exception parsing, presentational widgets, and the dashboard's
blocs are all tested (see `test/`) - the bloc tests use `bloc_test`
against fake repositories (a subclass overriding just the methods
under test, since repositories are plain classes rather than built
against an interface - `ApiClient`'s constructor still needs a real
`TokenStorage`/`Dio`, but nothing in a fake ever calls through it).
`ClientsBloc` and its screens don't have this treatment yet; the same
fake-subclass approach works there too.

## Notable pieces

- `lib/core/network/api_client.dart` - the Dio wrapper every repository
  goes through: attaches the bearer token, normalizes backend error
  bodies into `ApiException` (mirrors `frontend/src/lib/api/http.ts`).
- `lib/core/router/app_router.dart` - the single auth redirect guard
  (go_router `redirect`, driven by `AuthBloc`'s stream) that stands in
  for the web app's per-route `beforeLoad` guards.
- `lib/features/clients/` - the reference feature. Copy its shape for
  the next one.
