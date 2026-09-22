<?php

declare(strict_types=1);

namespace App\Database;

use Hyperdrive\Config\Config;
use PDO;

// Thin PDO wrapper around the Postgres connection. Resolved through
// Hyperdrive's container like any other service, so it's a singleton per
// worker process - fine for a database handle. Connects lazily, on the
// first pdo() call, rather than in the constructor - so building a
// Connection (e.g. as an unused constructor dependency) never requires a
// live database, only actually querying does.
class Connection
{
    private ?PDO $pdo = null;

    public function pdo(): PDO
    {
        if ($this->pdo === null) {
            $config = Config::get('database.pgsql');

            $dsn = sprintf(
                'pgsql:host=%s;port=%d;dbname=%s',
                $config['host'],
                $config['port'],
                $config['database'],
            );

            $this->pdo = new PDO($dsn, $config['username'], $config['password'], [
                PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
                PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
            ]);
        }

        return $this->pdo;
    }
}
