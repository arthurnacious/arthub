<?php

declare(strict_types=1);

use App\Http\Middleware\CorsMiddleware;
use Hyperdrive\Config\Config;
use Hyperdrive\Http\Middleware\RequestHandlerInterface;
use Hyperdrive\Http\Request;
use Hyperdrive\Http\Response;

it('adds CORS headers from config to the response', function () {
    Config::set('cors.allowed_origin', 'https://test.example');

    $finalHandler = new class implements RequestHandlerInterface {
        public function handle(Request $request): Response
        {
            return new Response('hello', 200);
        }
    };

    $response = (new CorsMiddleware)->handle(new Request, $finalHandler);

    expect($response->getStatusCode())->toBe(200)
        ->and($response->getContent())->toBe('hello')
        ->and($response->getHeaders())
        ->toMatchArray([
            'Access-Control-Allow-Origin' => 'https://test.example',
            'Access-Control-Allow-Methods' => 'GET, POST, PUT, PATCH, DELETE, OPTIONS',
            'Access-Control-Allow-Headers' => 'Content-Type, Authorization',
            'Access-Control-Allow-Credentials' => 'true',
        ]);
});

it('preserves headers already set on the response', function () {
    $finalHandler = new class implements RequestHandlerInterface {
        public function handle(Request $request): Response
        {
            return new Response('{}', 200, ['X-Custom' => 'value']);
        }
    };

    $response = (new CorsMiddleware)->handle(new Request, $finalHandler);

    expect($response->getHeaders())->toHaveKey('X-Custom', 'value');
});

it('echoes back whichever configured origin the request actually came from', function () {
    Config::set('cors.allowed_origin', 'http://localhost:3000,http://192.168.0.170:3000');

    $finalHandler = new class implements RequestHandlerInterface {
        public function handle(Request $request): Response
        {
            return new Response('hello', 200);
        }
    };

    $fromLan = (new CorsMiddleware)->handle(
        new Request(server: ['HTTP_ORIGIN' => 'http://192.168.0.170:3000']),
        $finalHandler,
    );
    $fromLocalhost = (new CorsMiddleware)->handle(
        new Request(server: ['HTTP_ORIGIN' => 'http://localhost:3000']),
        $finalHandler,
    );

    expect($fromLan->getHeaders()['Access-Control-Allow-Origin'])->toBe('http://192.168.0.170:3000')
        ->and($fromLocalhost->getHeaders()['Access-Control-Allow-Origin'])->toBe('http://localhost:3000');
});

it('falls back to the first configured origin for a request Origin not in the list', function () {
    Config::set('cors.allowed_origin', 'http://localhost:3000,http://192.168.0.170:3000');

    $finalHandler = new class implements RequestHandlerInterface {
        public function handle(Request $request): Response
        {
            return new Response('hello', 200);
        }
    };

    $response = (new CorsMiddleware)->handle(
        new Request(server: ['HTTP_ORIGIN' => 'https://evil.example']),
        $finalHandler,
    );

    expect($response->getHeaders()['Access-Control-Allow-Origin'])->toBe('http://localhost:3000');
});

it('falls back to the first configured origin when the request has no Origin header at all', function () {
    Config::set('cors.allowed_origin', 'http://localhost:3000,http://192.168.0.170:3000');

    $finalHandler = new class implements RequestHandlerInterface {
        public function handle(Request $request): Response
        {
            return new Response('hello', 200);
        }
    };

    $response = (new CorsMiddleware)->handle(new Request, $finalHandler);

    expect($response->getHeaders()['Access-Control-Allow-Origin'])->toBe('http://localhost:3000');
});
