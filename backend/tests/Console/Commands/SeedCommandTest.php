<?php

declare(strict_types=1);

use App\Console\Commands\SeedCommand;
use Symfony\Component\Console\Tester\CommandTester;

function run_seed_command(): CommandTester
{
    $tester = new CommandTester(new SeedCommand(test_connection()));
    $tester->execute([]);

    return $tester;
}

// Tests below mutate $_ENV directly, which the transaction rollback (see
// Tests\TestCase) doesn't touch - clean up so it can't leak into whatever
// test runs next.
afterEach(function () {
    unset($_ENV['ADMIN_EMAIL'], $_ENV['ADMIN_PASSWORD']);
});

it('creates the admin user from ADMIN_EMAIL/ADMIN_PASSWORD', function () {
    $_ENV['ADMIN_EMAIL'] = 'seed-admin@example.com';
    $_ENV['ADMIN_PASSWORD'] = 'seed-password';

    $tester = run_seed_command();

    expect($tester->getStatusCode())->toBe(0);

    $user = test_connection()->pdo()
        ->prepare('SELECT * FROM users WHERE email = :email')
        ->execute(['email' => 'seed-admin@example.com']);

    $row = test_connection()->pdo()
        ->query("SELECT * FROM users WHERE email = 'seed-admin@example.com'")
        ->fetch();

    expect($row)->not->toBeFalse()
        ->and($row['role'])->toBe('admin')
        ->and(password_verify('seed-password', $row['password_hash']))->toBeTrue();
});

it('does nothing if that email already exists', function () {
    create_test_user(email: 'already-here@example.com', password: 'original-password');

    $_ENV['ADMIN_EMAIL'] = 'already-here@example.com';
    $_ENV['ADMIN_PASSWORD'] = 'a-different-password';

    $tester = run_seed_command();

    expect($tester->getStatusCode())->toBe(0)
        ->and($tester->getDisplay())->toContain('already exists');

    $row = test_connection()->pdo()
        ->query("SELECT * FROM users WHERE email = 'already-here@example.com'")
        ->fetch();

    // Still the original password, not overwritten.
    expect(password_verify('original-password', $row['password_hash']))->toBeTrue();
});

it('fails with a clear message when ADMIN_EMAIL/ADMIN_PASSWORD are not set', function () {
    unset($_ENV['ADMIN_EMAIL'], $_ENV['ADMIN_PASSWORD']);

    $tester = run_seed_command();

    expect($tester->getStatusCode())->toBe(1)
        ->and($tester->getDisplay())->toContain('ADMIN_EMAIL');
});

it('seeds sample clients', function () {
    $_ENV['ADMIN_EMAIL'] = 'seed-admin@example.com';
    $_ENV['ADMIN_PASSWORD'] = 'seed-password';

    $tester = run_seed_command();

    expect($tester->getStatusCode())->toBe(0)
        ->and($tester->getDisplay())->toContain('Seeded 6 clients');

    $count = test_connection()->pdo()->query('SELECT COUNT(*) FROM clients')->fetchColumn();
    expect((int) $count)->toBe(6);
});

it('does nothing if clients already exist', function () {
    create_test_client(company: 'Already Here', email: 'already-here-client@test.com');

    $_ENV['ADMIN_EMAIL'] = 'seed-admin@example.com';
    $_ENV['ADMIN_PASSWORD'] = 'seed-password';

    $tester = run_seed_command();

    expect($tester->getStatusCode())->toBe(0)
        ->and($tester->getDisplay())->toContain('Clients already seeded');

    $count = test_connection()->pdo()->query('SELECT COUNT(*) FROM clients')->fetchColumn();
    expect((int) $count)->toBe(1);
});
