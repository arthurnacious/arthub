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
//
// cors.allowed_origin can be a single origin or a comma-separated list
// (e.g. "http://localhost:3000,http://192.168.0.170:3000" - the same dev
// machine reachable both as localhost and by its LAN IP, for testing on a
// phone on the same network). The response echoes back whichever
// configured origin the request actually came from (never a wildcard -
// that's incompatible with Access-Control-Allow-Credentials), falling
// back to the first configured origin when the request has none (e.g. a
// same-origin request, or a plain curl/test request with no Origin
// header at all).
class CorsMiddleware implements MiddlewareInterface
{
    public function handle(Request $request, RequestHandlerInterface $handler): Response
    {
        $response = $handler->handle($request);

        $allowedOrigins = array_values(array_filter(array_map(
            'trim',
            explode(',', Config::get('cors.allowed_origin', '*')),
        )));
        $requestOrigin = $request->server['HTTP_ORIGIN'] ?? null;
        $allowOrigin = in_array($requestOrigin, $allowedOrigins, true)
            ? $requestOrigin
            : ($allowedOrigins[0] ?? '*');

        $headers = array_merge($response->getHeaders(), [
            'Access-Control-Allow-Origin' => $allowOrigin,
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
