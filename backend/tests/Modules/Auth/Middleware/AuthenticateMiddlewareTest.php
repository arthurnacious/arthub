<?php

declare(strict_types=1);

use App\Modules\Auth\Middleware\AuthenticateMiddleware;
use Hyperdrive\Http\Middleware\RequestHandlerInterface;
use Hyperdrive\Http\Request;
use Hyperdrive\Http\Response;
use Hyperdrive\Security\JwtService;

function auth_middleware_passthrough_handler(): RequestHandlerInterface
{
    return new class implements RequestHandlerInterface {
        public function handle(Request $request): Response
        {
            // Echo back whatever the middleware attached, so tests can
            // assert on it.
            return Response::json(['auth_user' => $request->getAttribute('auth_user')]);
        }
    };
}

it('rejects a request with no Authorization header', function () {
    $response = (new AuthenticateMiddleware(new JwtService))->handle(new Request, auth_middleware_passthrough_handler());

    expect($response->getStatusCode())->toBe(401);
});

it('rejects a header that is not a Bearer token', function () {
    $request = new Request(server: ['HTTP_AUTHORIZATION' => 'Basic dXNlcjpwYXNz']);

    $response = (new AuthenticateMiddleware(new JwtService))->handle($request, auth_middleware_passthrough_handler());

    expect($response->getStatusCode())->toBe(401);
});

it('rejects an invalid token', function () {
    $request = new Request(server: ['HTTP_AUTHORIZATION' => 'Bearer not-a-real-token']);

    $response = (new AuthenticateMiddleware(new JwtService))->handle($request, auth_middleware_passthrough_handler());

    expect($response->getStatusCode())->toBe(401);
});

it('rejects a purpose-scoped token (e.g. a password reset token)', function () {
    $jwt = new JwtService;
    $resetToken = $jwt->encode(['sub' => 1, 'purpose' => 'password-reset']);
    $request = new Request(server: ['HTTP_AUTHORIZATION' => "Bearer {$resetToken}"]);

    $response = (new AuthenticateMiddleware($jwt))->handle($request, auth_middleware_passthrough_handler());

    expect($response->getStatusCode())->toBe(401);
});

it('accepts a valid session token and attaches its payload to the request', function () {
    $jwt = new JwtService;
    $token = $jwt->encode(['sub' => 42, 'email' => 'user@example.com', 'role' => 'admin']);
    $request = new Request(server: ['HTTP_AUTHORIZATION' => "Bearer {$token}"]);

    $response = (new AuthenticateMiddleware($jwt))->handle($request, auth_middleware_passthrough_handler());
    $payload = json_decode($response->getContent(), true)['auth_user'];

    expect($response->getStatusCode())->toBe(200)
        ->and($payload['sub'])->toBe(42)
        ->and($payload['email'])->toBe('user@example.com')
        ->and($payload['role'])->toBe('admin');
});

it('falls back to REDIRECT_HTTP_AUTHORIZATION when HTTP_AUTHORIZATION is missing', function () {
    $jwt = new JwtService;
    $token = $jwt->encode(['sub' => 1]);
    $request = new Request(server: ['REDIRECT_HTTP_AUTHORIZATION' => "Bearer {$token}"]);

    $response = (new AuthenticateMiddleware($jwt))->handle($request, auth_middleware_passthrough_handler());

    expect($response->getStatusCode())->toBe(200);
});
