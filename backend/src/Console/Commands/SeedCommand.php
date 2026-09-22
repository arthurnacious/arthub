<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\Database\Connection;
use PDO;
use Symfony\Component\Console\Attribute\AsCommand;
use Symfony\Component\Console\Command\Command;
use Symfony\Component\Console\Input\InputInterface;
use Symfony\Component\Console\Output\OutputInterface;

// `bin/console db:seed` - creates the very first admin user from
// ADMIN_EMAIL/ADMIN_PASSWORD in .env. Safe to run more than once: it does
// nothing if that email is already registered.
#[AsCommand(name: 'db:seed', description: 'Seed the initial admin user')]
class SeedCommand extends Command
{
    public function __construct(private readonly Connection $connection)
    {
        parent::__construct();
    }

    protected function execute(InputInterface $input, OutputInterface $output): int
    {
        $email = $_ENV['ADMIN_EMAIL'] ?? null;
        $password = $_ENV['ADMIN_PASSWORD'] ?? null;

        if (!$email || !$password) {
            $output->writeln('<error>Set ADMIN_EMAIL and ADMIN_PASSWORD in .env first.</error>');

            return Command::FAILURE;
        }

        $pdo = $this->connection->pdo();

        $exists = $pdo->prepare('SELECT 1 FROM users WHERE email = :email');
        $exists->execute(['email' => $email]);

        if ($exists->fetchColumn() !== false) {
            $output->writeln("<comment>{$email} already exists, skipping.</comment>");

            return Command::SUCCESS;
        }

        $statement = $pdo->prepare(<<<SQL
            INSERT INTO users (name, email, password_hash, role)
            VALUES (:name, :email, :password_hash, 'admin')
        SQL);

        $statement->execute([
            'name' => 'Admin',
            'email' => $email,
            'password_hash' => password_hash($password, PASSWORD_BCRYPT),
        ]);

        $output->writeln("<info>Created admin user:</info> {$email}");

        return Command::SUCCESS;
    }
}
