<?php

declare(strict_types=1);

use App\Database\Migrator;

// migrator_connection() (defined in Pest.php) is a second, independent
// connection to the same database, used instead of test_connection() here
// because Migrator manages its own transactions - see that function's
// docblock. This file cleans up explicitly rather than relying on a
// rollback.

function migrator_under_test(): Migrator
{
    return new Migrator(migrator_connection(), __DIR__ . '/../Fixtures/migrations');
}

afterEach(function () {
    $pdo = migrator_connection()->pdo();
    $pdo->exec('DROP TABLE IF EXISTS scratch_widgets');
    $pdo->exec('DROP TABLE IF EXISTS scratch_gadgets');
    $pdo->exec("DELETE FROM migrations WHERE migration LIKE '2099_%'");
});

it('runs every pending fixture migration and records them in a batch', function () {
    $ran = migrator_under_test()->migrate();

    expect($ran)->toBe([
        '2099_01_01_000001_create_scratch_widgets_table',
        '2099_01_01_000002_create_scratch_gadgets_table',
    ]);

    $pdo = migrator_connection()->pdo();
    expect($pdo->query("SELECT to_regclass('scratch_widgets')")->fetchColumn())->toBe('scratch_widgets')
        ->and($pdo->query("SELECT to_regclass('scratch_gadgets')")->fetchColumn())->toBe('scratch_gadgets');

    $batches = $pdo->query("SELECT DISTINCT batch FROM migrations WHERE migration LIKE '2099_%'")->fetchAll(PDO::FETCH_COLUMN);
    expect($batches)->toHaveCount(1);
});

it('does not re-run migrations that are already applied', function () {
    migrator_under_test()->migrate();

    $ranAgain = migrator_under_test()->migrate();

    expect($ranAgain)->toBe([]);
});

it('rolls back every migration in the most recent batch, in reverse order', function () {
    migrator_under_test()->migrate();

    $rolledBack = migrator_under_test()->rollback();

    expect($rolledBack)->toBe([
        '2099_01_01_000002_create_scratch_gadgets_table',
        '2099_01_01_000001_create_scratch_widgets_table',
    ]);

    $pdo = migrator_connection()->pdo();
    expect($pdo->query("SELECT to_regclass('scratch_widgets')")->fetchColumn())->toBeNull()
        ->and($pdo->query("SELECT to_regclass('scratch_gadgets')")->fetchColumn())->toBeNull();

    $remaining = $pdo->query("SELECT count(*) FROM migrations WHERE migration LIKE '2099_%'")->fetchColumn();
    expect((int) $remaining)->toBe(0);
});

// Deliberately not testing "rollback() with nothing to roll back" here:
// Migrator::rollback() rolls back whatever the latest batch is, with no
// concept of "which migrations directory" it belongs to. The real app
// migrations (from tests/Pest.php's own migrate() call) always occupy
// batch 1 in this shared arthub_test database/migrations table, so an
// empty-migrations-table state is never actually reachable here - and
// forcing it would mean either rolling back the real schema (breaking
// every other test) or a much heavier isolated-database test setup than
// this one behaviour is worth.
