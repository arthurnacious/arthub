<?php

declare(strict_types=1);

// Entry point for the OpenSwoole/Swoole drivers - unlike public/index.php
// (the traditional per-request front controller), this boots ONE
// long-lived process that runs its own event loop via listen(). Driver
// choice comes from .env so switching drivers never means editing code.
//
// Run with: php server.php

require __DIR__ . '/bootstrap.php';

use App\AppModule;
use Hyperdrive\Application\Hyperdrive;

$app = Hyperdrive::create(
    rootModule: AppModule::class,
    driver: $_ENV['DRIVER'] ?? 'openswoole',
    environment: $_ENV['APP_ENV'] ?? 'production',
    url: $_ENV['APP_URL'] ?? 'http://localhost:9501',
);

$app->boot();

$app->listen(
    port: (int) ($_ENV['APP_PORT'] ?? 9501),
    host: $_ENV['APP_HOST'] ?? '0.0.0.0',
);
