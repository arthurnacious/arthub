<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\Database\Migrator;
use Symfony\Component\Console\Attribute\AsCommand;
use Symfony\Component\Console\Command\Command;
use Symfony\Component\Console\Input\InputInterface;
use Symfony\Component\Console\Output\OutputInterface;

// `bin/console migrate` - runs every migration that hasn't been applied yet.
#[AsCommand(name: 'migrate', description: 'Run pending database migrations')]
class MigrateCommand extends Command
{
    public function __construct(private readonly Migrator $migrator)
    {
        parent::__construct();
    }

    protected function execute(InputInterface $input, OutputInterface $output): int
    {
        $ran = $this->migrator->migrate();

        if ($ran === []) {
            $output->writeln('<comment>Nothing to migrate.</comment>');

            return Command::SUCCESS;
        }

        foreach ($ran as $migration) {
            $output->writeln("<info>Migrated:</info> {$migration}");
        }

        return Command::SUCCESS;
    }
}
