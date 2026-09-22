<?php

declare(strict_types=1);

use App\Console\Commands\MigrateCommand;
use App\Database\Migrator;
use Symfony\Component\Console\Tester\CommandTester;

// See MigratorTest.php: fixture migrations + the separate
// migrator_connection(), not test_connection(). Every test leaves the
// fixture batch rolled back afterwards, so these two files don't interfere
// with each other regardless of run order.
afterEach(function () {
    $pdo = migrator_connection()->pdo();
    $pdo->exec('DROP TABLE IF EXISTS scratch_widgets');
    $pdo->exec('DROP TABLE IF EXISTS scratch_gadgets');
    $pdo->exec("DELETE FROM migrations WHERE migration LIKE '2099_%'");
});

it('runs pending migrations and reports each one', function () {
    $migrator = new Migrator(migrator_connection(), __DIR__ . '/../../Fixtures/migrations');
    $tester = new CommandTester(new MigrateCommand($migrator));

    $tester->execute([]);

    expect($tester->getStatusCode())->toBe(0)
        ->and($tester->getDisplay())->toContain('2099_01_01_000001_create_scratch_widgets_table')
        ->and($tester->getDisplay())->toContain('2099_01_01_000002_create_scratch_gadgets_table');
});

it('reports when there is nothing to migrate', function () {
    $migrator = new Migrator(migrator_connection(), __DIR__ . '/../../Fixtures/migrations');
    $migrator->migrate();

    $tester = new CommandTester(new MigrateCommand($migrator));
    $tester->execute([]);

    expect($tester->getStatusCode())->toBe(0)
        ->and($tester->getDisplay())->toContain('Nothing to migrate');
});
