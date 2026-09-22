<?php

declare(strict_types=1);

namespace App\Modules\Auth\Middleware;

use Hyperdrive\Http\Middleware\MiddlewareInterface;
use Hyperdrive\Http\Middleware\RequestHandlerInterface;
use Hyperdrive\Http\Request;
use Hyperdrive\Http\Response;
use Hyperdrive\Security\JwtService;
use Throwable;

// Protects a route behind a valid session JWT ("Authorization: Bearer
// <token>"). On success, stashes the decoded payload on the request so
// controllers can read it back with $request->getAttribute('auth_user').
class AuthenticateMiddleware implements MiddlewareInterface
{
    public function __construct(private readonly JwtService $jwt)
    {
    }

    public function handle(Request $request, RequestHandlerInterface $handler): Response
    {
        // Some SAPIs (Apache in particular) don't always populate
        // HTTP_AUTHORIZATION directly - REDIRECT_HTTP_AUTHORIZATION is the
        // common fallback.
        $header = $request->server['HTTP_AUTHORIZATION']
            ?? $request->server['REDIRECT_HTTP_AUTHORIZATION']
            ?? null;

        if ($header === null || !str_starts_with($header, 'Bearer ')) {
            return Response::json(['error' => 'Unauthenticated'], 401);
        }

        try {
            $payload = $this->jwt->verify(substr($header, 7));
        } catch (Throwable) {
            return Response::json(['error' => 'Unauthenticated'], 401);
        }

        // Purpose-scoped tokens (e.g. a password-reset token) must never be
        // usable as a session token.
        if (($payload['purpose'] ?? null) !== null) {
            return Response::json(['error' => 'Unauthenticated'], 401);
        }

        return $handler->handle($request->withAttribute('auth_user', $payload));
    }
}
