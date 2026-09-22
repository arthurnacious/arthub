# Arthub backend

API for Arthub: clients, projects, quotes and invoices. Built on
[hyperdrive/framework](https://github.com/arthurnacious/hyperdrive), running
under its traditional PHP ("Roadstar") driver behind a front controller -
no persistent server process, just a normal PHP-FPM/Apache/`php -S` setup.

## Status

This is phase 1: project scaffold, the Postgres migration system + CLI,
the full core schema, and admin login/password-reset. Clients, projects,
quotes and invoices don't have API endpoints yet - that's next.

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
