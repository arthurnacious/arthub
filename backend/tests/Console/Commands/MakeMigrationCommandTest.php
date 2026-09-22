<?php

declare(strict_types=1);

use App\Console\Commands\MakeMigrationCommand;
use Symfony\Component\Console\Tester\CommandTester;

function make_migration_scratch_dir(): string
{
    $dir = sys_get_temp_dir() . '/arthub_make_migration_test';

    if (!is_dir($dir)) {
        mkdir($dir);
    }

    return $dir;
}

afterEach(function () {
    $dir = make_migration_scratch_dir();

    foreach (glob($dir . '/*.php') as $file) {
        unlink($file);
    }
});

it('creates a timestamp-prefixed migration file with an up()/down() stub', function () {
    $tester = new CommandTester(new MakeMigrationCommand(make_migration_scratch_dir()));

    $tester->execute(['name' => 'create_widgets_table']);

    $files = glob(make_migration_scratch_dir() . '/*_create_widgets_table.php');
    expect($files)->toHaveCount(1);

    $contents = file_get_contents($files[0]);
    expect($contents)->toContain('extends Migration')
        ->and($contents)->toContain('public function up(): void')
        ->and($contents)->toContain('public function down(): void');

    expect(basename($files[0]))->toMatch('/^\d{4}_\d{2}_\d{2}_\d{6}_create_widgets_table\.php$/');
});

it('prints the created path', function () {
    $tester = new CommandTester(new MakeMigrationCommand(make_migration_scratch_dir()));

    $tester->execute(['name' => 'add_status_to_projects']);

    expect($tester->getDisplay())->toContain('add_status_to_projects');
});
