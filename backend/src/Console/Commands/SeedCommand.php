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
// ADMIN_EMAIL/ADMIN_PASSWORD in .env, then a handful of sample clients.
// Safe to run more than once: each step does nothing if its data already
// exists (an admin with that email, or any clients at all).
#[AsCommand(name: 'db:seed', description: 'Seed the initial admin user and sample clients')]
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

        $this->seedAdmin($pdo, $output, $email, $password);
        $this->seedClients($pdo, $output);

        return Command::SUCCESS;
    }

    private function seedAdmin(PDO $pdo, OutputInterface $output, string $email, string $password): void
    {
        $exists = $pdo->prepare('SELECT 1 FROM users WHERE email = :email');
        $exists->execute(['email' => $email]);

        if ($exists->fetchColumn() !== false) {
            $output->writeln("<comment>{$email} already exists, skipping.</comment>");

            return;
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
    }

    private function seedClients(PDO $pdo, OutputInterface $output): void
    {
        $count = (int) $pdo->query('SELECT COUNT(*) FROM clients')->fetchColumn();

        if ($count > 0) {
            $output->writeln('<comment>Clients already seeded, skipping.</comment>');

            return;
        }

        $clients = [
            ['Nimbus Retail', 'Dana Whitfield', 'dana@nimbusretail.test', 'active'],
            ['Harbour Digital', 'Priya Anand', 'priya@harbourdigital.test', 'active'],
            ['Cloud North Solutions', 'Marcus Ellery', 'marcus@cloudnorth.test', 'prospect'],
            ['Tidal Commerce', 'Sofia Reyes', 'sofia@tidalcommerce.test', 'active'],
            ['Rainforest Analytics', 'Owen Baptiste', 'owen@rainforestanalytics.test', 'inactive'],
            ['Granite Peak Studio', 'Leah Nakamura', 'leah@granitepeak.test', 'prospect'],
        ];

        $statement = $pdo->prepare(<<<SQL
            INSERT INTO clients (company_name, contact_name, email, status)
            VALUES (:company_name, :contact_name, :email, :status)
        SQL);

        foreach ($clients as [$company, $contact, $email, $status]) {
            $statement->execute([
                'company_name' => $company,
                'contact_name' => $contact,
                'email' => $email,
                'status' => $status,
            ]);
        }

        $output->writeln('<info>Seeded ' . count($clients) . ' clients.</info>');
    }
}
