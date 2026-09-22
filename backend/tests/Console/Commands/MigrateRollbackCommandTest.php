<?php

declare(strict_types=1);

use App\Console\Commands\MigrateRollbackCommand;
use App\Database\Migrator;
use Symfony\Component\Console\Tester\CommandTester;

// See MigratorTest.php for why this uses fixture migrations and the
// separate migrator_connection() rather than test_connection().
afterEach(function () {
    $pdo = migrator_connection()->pdo();
    $pdo->exec('DROP TABLE IF EXISTS scratch_widgets');
    $pdo->exec('DROP TABLE IF EXISTS scratch_gadgets');
    $pdo->exec("DELETE FROM migrations WHERE migration LIKE '2099_%'");
});

it('rolls back the last batch and reports each migration', function () {
    $migrator = new Migrator(migrator_connection(), __DIR__ . '/../../Fixtures/migrations');
    $migrator->migrate();

    $tester = new CommandTester(new MigrateRollbackCommand($migrator));
    $tester->execute([]);

    expect($tester->getStatusCode())->toBe(0)
        ->and($tester->getDisplay())->toContain('2099_01_01_000001_create_scratch_widgets_table')
        ->and($tester->getDisplay())->toContain('2099_01_01_000002_create_scratch_gadgets_table');

    $pdo = migrator_connection()->pdo();
    expect($pdo->query("SELECT to_regclass('scratch_widgets')")->fetchColumn())->toBeNull();
});
