<?php

declare(strict_types=1);

namespace App\Console\Commands;

use Symfony\Component\Console\Attribute\AsCommand;
use Symfony\Component\Console\Command\Command;
use Symfony\Component\Console\Input\InputArgument;
use Symfony\Component\Console\Input\InputInterface;
use Symfony\Component\Console\Output\OutputInterface;

// `bin/console make:migration create_widgets_table` - scaffolds a new,
// empty migration file with a timestamp prefix so files sort (and
// therefore run) in creation order.
#[AsCommand(name: 'make:migration', description: 'Create a new migration file')]
class MakeMigrationCommand extends Command
{
    public function __construct(private readonly string $migrationsPath)
    {
        parent::__construct();
    }

    protected function configure(): void
    {
        $this->addArgument('name', InputArgument::REQUIRED, 'Snake_case name, e.g. create_widgets_table');
    }

    protected function execute(InputInterface $input, OutputInterface $output): int
    {
        $name = (string) $input->getArgument('name');
        $timestamp = date('Y_m_d_His');
        $filename = "{$timestamp}_{$name}.php";
        $path = $this->migrationsPath . '/' . $filename;

        file_put_contents($path, $this->stub());

        $output->writeln("<info>Created:</info> database/migrations/{$filename}");

        return Command::SUCCESS;
    }

    private function stub(): string
    {
        return <<<'PHP'
            <?php

            declare(strict_types=1);

            use App\Database\Migration;

            // $pdo is injected by Migrator when this file is required - see
            // App\Database\Migrator::instantiate().
            return new class ($pdo) extends Migration {
                public function up(): void
                {
                    $this->pdo->exec(<<<SQL
                        -- TODO
                    SQL);
                }

                public function down(): void
                {
                    $this->pdo->exec(<<<SQL
                        -- TODO
                    SQL);
                }
            };

            PHP;
    }
}
