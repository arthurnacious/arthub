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
