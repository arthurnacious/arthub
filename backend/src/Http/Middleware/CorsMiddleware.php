<?php

declare(strict_types=1);

namespace App\Http\Middleware;

use Hyperdrive\Config\Config;
use Hyperdrive\Http\Middleware\MiddlewareInterface;
use Hyperdrive\Http\Middleware\RequestHandlerInterface;
use Hyperdrive\Http\Request;
use Hyperdrive\Http\Response;

// Allows the TanStack Start frontend (a different origin) to call this
// API, including its OPTIONS preflight request - registered as global
// middleware in config/middleware.php so it runs for every request.
class CorsMiddleware implements MiddlewareInterface
{
    public function handle(Request $request, RequestHandlerInterface $handler): Response
    {
        $response = $handler->handle($request);

        $headers = array_merge($response->getHeaders(), [
            'Access-Control-Allow-Origin' => Config::get('cors.allowed_origin', '*'),
            'Access-Control-Allow-Methods' => 'GET, POST, PUT, PATCH, DELETE, OPTIONS',
            'Access-Control-Allow-Headers' => 'Content-Type, Authorization',
            'Access-Control-Allow-Credentials' => 'true',
        ]);

        // Response has no header-mutation method, so rebuild it with the
        // extra headers merged in (same pattern the framework's own docs
        // use for middleware that needs to add response headers).
        return new Response($response->getContent(), $response->getStatusCode(), $headers, $response->getCookies());
    }
}
