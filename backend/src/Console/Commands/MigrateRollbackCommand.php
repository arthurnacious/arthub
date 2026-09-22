<?php

declare(strict_types=1);

namespace App\Console\Commands;

use App\Database\Migrator;
use Symfony\Component\Console\Attribute\AsCommand;
use Symfony\Component\Console\Command\Command;
use Symfony\Component\Console\Input\InputInterface;
use Symfony\Component\Console\Output\OutputInterface;

// `bin/console migrate:rollback` - undoes the most recent batch of
// migrations (everything that was applied together by one `migrate` run).
#[AsCommand(name: 'migrate:rollback', description: 'Roll back the last batch of migrations')]
class MigrateRollbackCommand extends Command
{
    public function __construct(private readonly Migrator $migrator)
    {
        parent::__construct();
    }

    protected function execute(InputInterface $input, OutputInterface $output): int
    {
        $rolledBack = $this->migrator->rollback();

        if ($rolledBack === []) {
            $output->writeln('<comment>Nothing to roll back.</comment>');

            return Command::SUCCESS;
        }

        foreach ($rolledBack as $migration) {
            $output->writeln("<info>Rolled back:</info> {$migration}");
        }

        return Command::SUCCESS;
    }
}
