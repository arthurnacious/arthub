<?php

declare(strict_types=1);

use App\Database\Connection;
use App\Database\Migrator;
use App\Modules\Activity\Listeners\ActivityLogListener;
use Hyperdrive\Container\Container;
use Hyperdrive\Events\EventDispatcher;
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
 * A real EventDispatcher wired the same way Hyperdrive wires one at boot -
 * except pointed at test_connection() (so anything ActivityLogListener
 * writes lands in this test's transaction and gets rolled back with
 * everything else) instead of a fresh Connection. Controllers/services
 * under test take an EventDispatcher in their constructor now instead of
 * an ActivityLogger directly, so this is what test helpers like
 * make_clients_controller() pass in.
 */
function make_event_dispatcher(): EventDispatcher
{
    $container = new Container();
    $container->instance(Connection::class, test_connection());

    $dispatcher = new EventDispatcher($container);
    $dispatcher->registerListenerClass(ActivityLogListener::class);

    return $dispatcher;
}

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

/**
 * Inserts a client row and returns its id (a UUID string).
 */
function create_test_client(
    string $company = 'Test Co',
    string $contact = 'Test Contact',
    string $email = 'client@example.com',
    string $status = 'active',
): string {
    $statement = test_connection()->pdo()->prepare(<<<SQL
        INSERT INTO clients (company_name, contact_name, email, status)
        VALUES (:company_name, :contact_name, :email, :status)
        RETURNING id
    SQL);

    $statement->execute([
        'company_name' => $company,
        'contact_name' => $contact,
        'email' => $email,
        'status' => $status,
    ]);

    return $statement->fetchColumn();
}

/**
 * Inserts a project row (creating a client for it if none is given) and
 * returns its id (a UUID string).
 */
function create_test_project(
    ?string $clientId = null,
    string $name = 'Test Project',
    string $category = 'general',
    ?string $url = null,
    string $status = 'active',
): string {
    $clientId ??= create_test_client(email: 'project-client-' . bin2hex(random_bytes(4)) . '@example.com');

    $statement = test_connection()->pdo()->prepare(<<<SQL
        INSERT INTO projects (client_id, name, category, url, status)
        VALUES (:client_id, :name, :category, :url, :status)
        RETURNING id
    SQL);

    $statement->execute([
        'client_id' => $clientId,
        'name' => $name,
        'category' => $category,
        'url' => $url,
        'status' => $status,
    ]);

    return $statement->fetchColumn();
}

/**
 * Inserts a quote row (creating a project for it if none is given) and
 * returns its id (a UUID string). No line items - tests that need them
 * insert into quote_lines directly, or go through the controller.
 */
function create_test_quote(
    ?string $projectId = null,
    string $status = 'draft',
    ?string $discountType = null,
    float $discountValue = 0,
): string {
    $projectId ??= create_test_project();

    $pdo = test_connection()->pdo();
    $sequence = (int) $pdo->query("SELECT nextval('quote_number_seq')")->fetchColumn();
    $quoteNumber = sprintf('Q-%04d', $sequence);

    $statement = $pdo->prepare(<<<SQL
        INSERT INTO quotes (project_id, quote_number, status, discount_type, discount_value)
        VALUES (:project_id, :quote_number, :status, :discount_type, :discount_value)
        RETURNING id
    SQL);

    $statement->execute([
        'project_id' => $projectId,
        'quote_number' => $quoteNumber,
        'status' => $status,
        'discount_type' => $discountType,
        'discount_value' => $discountValue,
    ]);

    return $statement->fetchColumn();
}
