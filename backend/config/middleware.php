<?php

declare(strict_types=1);

use App\Http\Middleware\CorsMiddleware;

// Global middleware - runs on every request, before any route or
// controller-specific middleware.
return [
    'global' => [
        CorsMiddleware::class,
    ],
];
