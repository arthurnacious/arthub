<?php

declare(strict_types=1);

// Postgres connection settings, read from the environment (.env is loaded
// by bootstrap.php before Hyperdrive's Config ever reads this file).
return [
    'pgsql' => [
        'host' => $_ENV['DB_HOST'] ?? '127.0.0.1',
        'port' => (int) ($_ENV['DB_PORT'] ?? 5432),
        'database' => $_ENV['DB_DATABASE'] ?? 'arthub',
        'username' => $_ENV['DB_USERNAME'] ?? 'arthub',
        'password' => $_ENV['DB_PASSWORD'] ?? '',
    ],
];
