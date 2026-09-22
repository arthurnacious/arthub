<?php

declare(strict_types=1);

require __DIR__ . '/../bootstrap.php';

use App\AppModule;
use Hyperdrive\Application\Hyperdrive;
use Hyperdrive\Http\Request;

$environment = $_ENV['APP_ENV'] ?? 'production';

$app = Hyperdrive::create(
    rootModule: AppModule::class,
    driver: 'roadstar', // traditional PHP, fronted by php -S or Apache/Nginx+FPM
    environment: $environment,
    url: $_ENV['APP_URL'] ?? 'http://localhost:9501',
);

$app->boot();

$request = Request::createFromGlobals();
$response = $app->getDriver()->handleRequest($request);
$response->send();
