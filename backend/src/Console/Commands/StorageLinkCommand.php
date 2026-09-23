<?php

declare(strict_types=1);

namespace App\Console\Commands;

use Symfony\Component\Console\Attribute\AsCommand;
use Symfony\Component\Console\Command\Command;
use Symfony\Component\Console\Input\InputInterface;
use Symfony\Component\Console\Output\OutputInterface;

// `bin/console storage:link` - Laravel-style symlink from public/storage to
// storage/app, so uploaded files (project images, ...) are reachable at
// {APP_URL}/storage/... once the driver's static handler is enabled
// (see config/server.php).
#[AsCommand(name: 'storage:link', description: 'Symlink public/storage to storage/app')]
class StorageLinkCommand extends Command
{
    public function __construct(private readonly string $basePath)
    {
        parent::__construct();
    }

    protected function execute(InputInterface $input, OutputInterface $output): int
    {
        $target = $this->basePath . '/storage/app';
        $link = $this->basePath . '/public/storage';

        if (!is_dir($target) && !mkdir($target, 0755, true) && !is_dir($target)) {
            $output->writeln("<error>Could not create {$target}</error>");

            return Command::FAILURE;
        }

        if (is_link($link) || file_exists($link)) {
            $output->writeln('<comment>public/storage already exists, skipping.</comment>');

            return Command::SUCCESS;
        }

        if (!symlink($target, $link)) {
            $output->writeln("<error>Could not symlink {$link} -> {$target}</error>");

            return Command::FAILURE;
        }

        $output->writeln("<info>Linked:</info> public/storage -> storage/app");

        return Command::SUCCESS;
    }
}
