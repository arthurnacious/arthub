<?php

declare(strict_types=1);

// Shared bootstrap for every entrypoint (HTTP front controller, CLI).
// Loads Composer's autoloader, then reads .env into $_ENV/$_SERVER so
// config/*.php files (which read $_ENV) see real values.

require __DIR__ . '/vendor/autoload.php';

Dotenv\Dotenv::createImmutable(__DIR__)->safeLoad();
