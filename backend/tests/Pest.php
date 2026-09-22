<?php

declare(strict_types=1);

use App\Database\Connection;
use App\Database\Migrator;
use Tests\TestCase;

/*
|--------------------------------------------------------------------------
| Test environment
|--------------------------------------------------------------------------
|
| Loaded here, at the top of Pest.php, rather than via phpunit.xml's
| `bootstrap` attribute: Pest's own BootFiles bootstrapper requires this
| file before PHPUnit's Application ever processes phpunit.xml, so
| anything wired through `bootstrap` runs too late for tests/Pest.php's
| own top-level code (like the migrate() call below) to see it.
|
*/

Dotenv\Dotenv::createImmutable(__DIR__ . '/..', '.env.testing')->safeLoad();

/*
|--------------------------------------------------------------------------
| Test database setup
|--------------------------------------------------------------------------
|
| Every test gets the same Connection (so their queries land in the same
| transaction), and every test runs inside a transaction that's rolled
| back afterwards - see Tests\TestCase - so tests never leave data behind
| for the next test, and can run in any order. The schema is brought up to
| date once, before the first test, against the dedicated arthub_test
| database.
|
*/

function test_connection(): Connection
{
    static $connection = null;

    return $connection ??= new Connection();
}

/**
 * A second, independent connection to the same database, for tests that
 * exercise Migrator (or anything else that manages its own transactions):
 * Migrator's own beginTransaction()/commit() per migration would conflict
 * with the outer transaction Tests\TestCase already wraps every test in
 * ("there is already an active transaction") if it shared test_connection().
 * Used by tests/Database/MigratorTest.php and the migrate/rollback command
 * tests - they clean up explicitly rather than relying on a rollback.
 */
function migrator_connection(): Connection
{
    static $connection = null;

    return $connection ??= new Connection();
}

(new Migrator(test_connection(), __DIR__ . '/../database/migrations'))->migrate();

uses(TestCase::class)->in(__DIR__);

/*
|--------------------------------------------------------------------------
| Shared fixtures
|--------------------------------------------------------------------------
*/

/**
 * Inserts a user row (inside the current test's transaction, so it's rolled
 * back like everything else) and returns its id (a UUID string).
 */
function create_test_user(
    string $name = 'Test User',
    string $email = 'test@example.com',
    string $password = 'password123',
    string $role = 'admin',
): string {
    $statement = test_connection()->pdo()->prepare(<<<SQL
        INSERT INTO users (name, email, password_hash, role)
        VALUES (:name, :email, :password_hash, :role)
        RETURNING id
    SQL);

    $statement->execute([
        'name' => $name,
        'email' => $email,
        'password_hash' => password_hash($password, PASSWORD_BCRYPT),
        'role' => $role,
    ]);

    return $statement->fetchColumn();
}
