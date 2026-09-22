<?php

declare(strict_types=1);

// Front controller for the traditional PHP-FPM/Apache/Nginx/`php -S`
// deployment model - one request in, one response out, per invocation.
// We run OpenSwoole day to day (see server.php), so this is kept as a
// fallback deployment option rather than the primary way to run the app;
// unlike server.php, the driver here is intentionally always 'roadstar',
// since a front controller only makes sense for the per-request driver.

require __DIR__ . '/../bootstrap.php';

use App\AppModule;
use Hyperdrive\Application\Hyperdrive;
use Hyperdrive\Http\Request;

$environment = $_ENV['APP_ENV'] ?? 'production';

$app = Hyperdrive::create(
    rootModule: AppModule::class,
    driver: 'roadstar',
    environment: $environment,
    url: $_ENV['APP_URL'] ?? 'http://localhost:9501',
);

$app->boot();

$request = Request::createFromGlobals();
$response = $app->getDriver()->handleRequest($request);
$response->send();
