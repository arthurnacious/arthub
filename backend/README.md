# Arthub backend

API for Arthub: clients, projects, quotes and invoices. Built on
[hyperdrive/framework](https://github.com/arthurnacious/hyperdrive), running
day to day under its OpenSwoole driver (`server.php`, driver chosen via
`DRIVER` in `.env`) - a long-lived coroutine server, not a per-request
process. `public/index.php` (the traditional PHP-FPM/Apache/`php -S` front
controller) is kept as a fallback deployment option, always hardcoded to
the `roadstar` driver regardless of `.env`.

Requires **Postgres 18+** - primary keys are UUIDv7, generated natively by
Postgres 18's `uuidv7()` (no extension needed). UUIDv7 rather than a plain
integer because it's time-ordered (unlike UUIDv4), so it stays
index-friendly like a normal incrementing key while still being safe to
expose and impossible to enumerate.

## Status

This is phase 1: project scaffold, the Postgres migration system + CLI,
the full core schema, admin login/password-reset, and a Pest test suite
covering all of it. Clients, projects, quotes and invoices don't have API
endpoints yet - that's next.

## Setup

1. Install dependencies:

   ```bash
   composer install
   ```

2. Create a Postgres role and database (adjust names/password to taste,
   just keep them in sync with `.env`):

   ```bash
   sudo -u postgres psql -c "CREATE ROLE arthub WITH LOGIN PASSWORD 'change-me';"
   sudo -u postgres psql -c "CREATE DATABASE arthub OWNER arthub;"
   ```

3. Copy the environment file and fill in real values (DB password, a
   random `JWT_SECRET`, SMTP credentials, your own admin email/password):

   ```bash
   cp .env.example .env
   ```

4. Run the migrations, then seed the first admin user:

   ```bash
   bin/console migrate
   bin/console db:seed
   ```

5. Serve it locally:

   ```bash
   php server.php               # OpenSwoole, long-running - see DRIVER in .env
   # or, for the traditional front-controller model instead:
   php -S localhost:9501 public/index.php
   ```

## The CLI (`bin/console`)

Laravel-`artisan`-style, built on Symfony Console (already a dependency
of hyperdrive/framework):

- `bin/console migrate` - run every migration that hasn't been applied yet.
- `bin/console migrate:rollback` - undo the most recent batch of migrations.
- `bin/console make:migration <name>` - scaffold a new, empty migration file.
- `bin/console db:seed` - create the admin user from `ADMIN_EMAIL`/`ADMIN_PASSWORD`.

Migrations live in `database/migrations/`, run raw SQL against Postgres via
PDO (no schema-builder DSL), and are tracked in a `migrations` table the
migrator creates automatically.

## Auth

`POST /auth/login`, `POST /auth/forgot-password`, `POST /auth/reset-password`.
Login returns a JWT; send it back as `Authorization: Bearer <token>` on
protected routes. Password reset tokens are short-lived, self-contained
JWTs (no database table needed) - see `App\Modules\Auth\Services\AuthService`.

## Activity log

Every interesting backend action writes a row to `activity_log` via
`App\Support\ActivityLogger`. `GET /activity-log` (requires auth) returns
the most recent 100 entries - this is what feeds the frontend's
terminal-style log panel.

## Testing

This project is TDD from here on - write the test first. Uses
[Pest](https://pestphp.com/), same as hyperdrive/framework itself.

```bash
composer test
```

Tests always run against a **separate** database (`arthub_test` by
default), never the real dev database:

1. Create it once: `sudo -u postgres psql -c "CREATE DATABASE arthub_test OWNER arthub;"`
2. `cp .env.testing.example .env.testing` and fill in real DB credentials.

`tests/Pest.php` migrates that database automatically before the first
test runs, and every test executes inside a Postgres transaction that's
rolled back afterwards (`Tests\TestCase`, bound via `uses()->in()` -
**not** bare `beforeEach()`/`afterEach()` calls in `Pest.php`, which look
like they'd apply globally but don't: Pest registers those under
`Pest.php`'s own filename, and each test looks its hooks up by its own
filename, so the two never actually connect unless done through
`uses()->in()`, which does the proper per-file wiring). So tests never
leave data behind for the next one and can run in any order.

One exception: `App\Database\Migrator` manages its own transactions
(one per migration), which would conflict with that outer transaction
("there is already an active transaction"). Tests that exercise it
(`tests/Database/MigratorTest.php`, the `migrate`/`migrate:rollback`
command tests) use a second, independent connection
(`migrator_connection()` in `Pest.php`) against fixture migrations in
`tests/Fixtures/migrations/` instead, and clean up explicitly.
