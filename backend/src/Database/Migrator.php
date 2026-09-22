<?php

declare(strict_types=1);

namespace App\Database;

use PDO;
use RuntimeException;
use Throwable;

// Small Postgres migration runner, Laravel-inspired but intentionally
// minimal: no schema-builder DSL, migrations just run raw SQL. Tracks
// what's been applied in a `migrations` table (created automatically) so
// `migrate` only runs what's new, and `migrate:rollback` can undo the
// last batch - the group of migrations that were applied together by one
// `migrate` run.
class Migrator
{
    private const TABLE = 'migrations';

    public function __construct(
        private readonly Connection $connection,
        private readonly string $migrationsPath,
    ) {
    }

    /**
     * @return string[] names of the migrations that were run, oldest first
     */
    public function migrate(): array
    {
        $this->ensureMigrationsTableExists();

        $pending = array_values(array_diff($this->migrationFiles(), $this->appliedMigrations()));

        if ($pending === []) {
            return [];
        }

        $batch = $this->nextBatchNumber();

        foreach ($pending as $file) {
            $this->runMigration($file, $batch);
        }

        return $pending;
    }

    /**
     * @return string[] names of the migrations that were rolled back, newest first
     */
    public function rollback(): array
    {
        $this->ensureMigrationsTableExists();

        $batch = $this->currentBatchNumber();

        if ($batch === null) {
            return [];
        }

        $pdo = $this->connection->pdo();

        $statement = $pdo->prepare(
            'SELECT migration FROM ' . self::TABLE . ' WHERE batch = :batch ORDER BY migration DESC'
        );
        $statement->execute(['batch' => $batch]);
        $files = $statement->fetchAll(PDO::FETCH_COLUMN);

        foreach ($files as $file) {
            $this->reverseMigration($file);
        }

        return $files;
    }

    private function runMigration(string $file, int $batch): void
    {
        $migration = $this->instantiate($file);
        $pdo = $this->connection->pdo();

        $pdo->beginTransaction();

        try {
            $migration->up();

            $statement = $pdo->prepare(
                'INSERT INTO ' . self::TABLE . ' (migration, batch, ran_at) VALUES (:migration, :batch, NOW())'
            );
            $statement->execute(['migration' => $file, 'batch' => $batch]);

            $pdo->commit();
        } catch (Throwable $e) {
            $pdo->rollBack();

            throw new RuntimeException("Migration {$file} failed: {$e->getMessage()}", previous: $e);
        }
    }

    private function reverseMigration(string $file): void
    {
        $migration = $this->instantiate($file);
        $pdo = $this->connection->pdo();

        $pdo->beginTransaction();

        try {
            $migration->down();

            $statement = $pdo->prepare('DELETE FROM ' . self::TABLE . ' WHERE migration = :migration');
            $statement->execute(['migration' => $file]);

            $pdo->commit();
        } catch (Throwable $e) {
            $pdo->rollBack();

            throw new RuntimeException("Rolling back {$file} failed: {$e->getMessage()}", previous: $e);
        }
    }

    private function instantiate(string $file): Migration
    {
        // Exposed as a local variable so the required file's own
        // `return new class ($pdo) extends Migration { ... }` can see it -
        // PHP `require` inherits the variable scope of the including line.
        $pdo = $this->connection->pdo();

        $migration = require $this->migrationsPath . '/' . $file . '.php';

        if (!$migration instanceof Migration) {
            throw new RuntimeException("Migration file {$file}.php must return a Migration instance");
        }

        return $migration;
    }

    /** @return string[] sorted filenames (without .php), oldest first */
    private function migrationFiles(): array
    {
        $files = glob($this->migrationsPath . '/*.php') ?: [];
        $names = array_map(static fn (string $path): string => basename($path, '.php'), $files);
        sort($names);

        return $names;
    }

    /** @return string[] */
    private function appliedMigrations(): array
    {
        return $this->connection->pdo()->query('SELECT migration FROM ' . self::TABLE)->fetchAll(PDO::FETCH_COLUMN);
    }

    private function nextBatchNumber(): int
    {
        return ($this->currentBatchNumber() ?? 0) + 1;
    }

    private function currentBatchNumber(): ?int
    {
        $max = $this->connection->pdo()->query('SELECT MAX(batch) FROM ' . self::TABLE)->fetchColumn();

        return $max !== null ? (int) $max : null;
    }

    private function ensureMigrationsTableExists(): void
    {
        $this->connection->pdo()->exec(<<<SQL
            CREATE TABLE IF NOT EXISTS migrations (
                id SERIAL PRIMARY KEY,
                migration VARCHAR(255) NOT NULL UNIQUE,
                batch INTEGER NOT NULL,
                ran_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        SQL);
    }
}
