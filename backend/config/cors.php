<?php

declare(strict_types=1);

// Read by App\Http\Middleware\CorsMiddleware. The frontend runs on a
// different origin (TanStack Start's own dev server), so the API needs to
// explicitly allow it. A single origin or a comma-separated list - see
// CorsMiddleware for how a list is matched against the request's own
// Origin header.
return [
    'allowed_origin' => $_ENV['CORS_ALLOWED_ORIGIN'] ?? 'http://localhost:3000',
];
