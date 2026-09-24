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
- **Navigation** - a drawer mirroring the web frontend's sidebar
  (dashboard/clients/projects/quotes/invoices/activity), with an
  `AuthBloc`-driven redirect guard (go_router) so every screen requires a
  session, same as every route on the web app.
- **Clients** - the one fully-built-out feature: a searched,
  infinite-scroll list backed by `GET /clients`. Read-only for now (no
  create/update/delete forms yet).

Everything else in the drawer (dashboard/projects/quotes/invoices/activity)
is a placeholder screen. Follow `lib/features/clients/`'s
`data/` → `bloc/` → `presentation/` split to build one out for real - see
the corresponding controller under `../backend/src/Modules/` for the API
shape, and the equivalent page under `../frontend/src/routes/` for what
it should end up looking like.

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

Model/exception parsing and presentational widgets are unit/widget
tested (see `test/`). Nothing exercises `ApiClient`/the repositories yet -
they're plain classes rather than built against an interface, so doing
that properly means introducing a repository interface first (an
abstract `ClientsRepository` a fake can implement) rather than mocking
`Dio` directly.

## Notable pieces

- `lib/core/network/api_client.dart` - the Dio wrapper every repository
  goes through: attaches the bearer token, normalizes backend error
  bodies into `ApiException` (mirrors `frontend/src/lib/api/http.ts`).
- `lib/core/router/app_router.dart` - the single auth redirect guard
  (go_router `redirect`, driven by `AuthBloc`'s stream) that stands in
  for the web app's per-route `beforeLoad` guards.
- `lib/features/clients/` - the reference feature. Copy its shape for
  the next one.
