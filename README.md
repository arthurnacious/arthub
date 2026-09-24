# Arthub

A self-hosted CRM for a freelancer to run their own client work: companies
(clients) → projects → quotes → invoices, with discounts, line items, and a
terminal-style activity log of everything that happens. Built by one person,
for that same person's own freelance business - not a multi-tenant SaaS.

It's also the first real application built on top of
[`hyperdrive/framework`](https://github.com/arthurnacious/hyperdrive), a
NestJS-inspired PHP framework - so it doubles as a proving ground for that
framework in a non-trivial, real-world app.

The frontend leans hard into a dark, retro-terminal aesthetic: a bordered
panel-grid dashboard, a monospace font, vim-style keyboard navigation
(`j`/`k`/`g`/`G`/`h`/`l`), and a `:`-prefixed command bar (`:ls`, `:cd
<route>`) for getting around without touching the mouse.

## Stack

| | |
|---|---|
| **Backend** | PHP 8.4+, [hyperdrive/framework](https://github.com/arthurnacious/hyperdrive), Postgres 18+ (UUIDv7 primary keys), OpenSwoole, PHPMailer, [Pest](https://pestphp.com/) |
| **Frontend** | [TanStack Start](https://tanstack.com/start) (React), TanStack Router/Query/Store, Tailwind CSS v4, Vitest |
| **App** | Flutter, flutter_bloc, go_router - see app/README.md (scaffold; one feature built out) |

## Layout

```
backend/    the API - see backend/README.md
frontend/   the dashboard UI - see frontend/README.md
app/        Flutter client for the same API - see app/README.md
```

The two are independent projects (separate dependency trees, separate test
suites) that happen to live in one repo. Each has its own README with full
setup instructions; this one is just the map.

## Quick start

1. **Backend** (`cd backend`): `composer install`, create a Postgres
   database, `cp .env.example .env` and fill it in, `bin/console migrate`,
   `bin/console db:seed`, then `php server.php` to serve it on
   `localhost:9501`. Full details in [`backend/README.md`](backend/README.md).
2. **Frontend** (`cd frontend`): `bun install`, `cp .env.example .env`,
   `bun --bun run dev` to serve it on `localhost:3000`. Full details in
   [`frontend/README.md`](frontend/README.md).

Both need to be running at once for the dashboard to eventually talk to real
data - see Status below for how much of that wiring actually exists yet.

## Status

**Backend**: the Postgres schema, migration system + CLI, admin
login/password-reset, and a full activity log are done and tested (TDD from
early on - see `backend/README.md`). Clients/projects/quotes/invoices don't
have API endpoints yet.

**Frontend**: the terminal dashboard shell is built - sidebar nav, top bar
(theme picker, notifications, user menu), the vim keybindings and command
bar, a Matrix-rain background, and a panel-grid dashboard - but it's still
running on mock data. No login flow yet, so nothing is wired to the real
backend.

**Next up**: CRUD endpoints for clients/projects/quotes/invoices on the
backend, a login flow on the frontend, then connecting the two. A future
"thoughts" feature (posts/comments/claps/views, separate from the CRM side)
is deferred indefinitely.

## Conventions

- Git: create a branch, commit there, merge to `main` - never commit
  directly on `main`.
- Commit messages: `<type> | <message>` (e.g. `feat | add invoice line
  items`), not Conventional Commits' `type: message`.
- TDD on both sides: write the test first.
