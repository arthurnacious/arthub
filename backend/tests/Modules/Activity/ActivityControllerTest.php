<?php

declare(strict_types=1);

use App\Modules\Activity\ActivityController;
use App\Support\ActivityLogger;
use Hyperdrive\Http\Request;

function make_activity_controller(): ActivityController
{
    return new ActivityController(test_connection());
}

it('lists recent activity, most recent first', function () {
    $logger = new ActivityLogger(test_connection());
    $logger->log('client.created', 'First event');
    $logger->log('client.updated', 'Second event');

    $response = make_activity_controller()->index(new Request());
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['data'][0]['description'])->toBe('Second event')
        ->and($body['data'][1]['description'])->toBe('First event');
});

it('returns an empty list when nothing has happened yet', function () {
    $response = make_activity_controller()->index(new Request());
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['data'])->toBe([]);
});

it('includes pagination metadata, defaulting to a large page size', function () {
    $logger = new ActivityLogger(test_connection());
    $logger->log('client.created', 'Event one');
    $logger->log('client.created', 'Event two');

    $response = make_activity_controller()->index(new Request());
    $body = json_decode($response->getContent(), true);

    expect($body['page'])->toBe(1)
        ->and($body['pageSize'])->toBe(100)
        ->and($body['total'])->toBe(2);
});

it('paginates when a smaller pageSize is requested', function () {
    $logger = new ActivityLogger(test_connection());
    $logger->log('client.created', 'Event one');
    $logger->log('client.created', 'Event two');
    $logger->log('client.created', 'Event three');

    $response = make_activity_controller()->index(
        new Request(query: ['page' => '2', 'pageSize' => '2']),
    );
    $body = json_decode($response->getContent(), true);

    expect($body['data'])->toHaveCount(1)
        ->and($body['total'])->toBe(3)
        ->and($body['page'])->toBe(2)
        ->and($body['pageSize'])->toBe(2);
});

it('searches by description or action', function () {
    $logger = new ActivityLogger(test_connection());
    $logger->log('client.created', 'Created client Nimbus Retail');
    $logger->log('auth.login', 'Admin logged in');

    $response = make_activity_controller()->index(
        new Request(query: ['q' => 'nimbus']),
    );
    $body = json_decode($response->getContent(), true);

    expect($body['data'])->toHaveCount(1)
        ->and($body['data'][0]['description'])->toBe('Created client Nimbus Retail');
});

it('matches a search term against the action too', function () {
    $logger = new ActivityLogger(test_connection());
    $logger->log('client.created', 'Created client Nimbus Retail');
    $logger->log('auth.login', 'Admin logged in');

    $response = make_activity_controller()->index(
        new Request(query: ['q' => 'auth.login']),
    );
    $body = json_decode($response->getContent(), true);

    expect($body['data'])->toHaveCount(1)
        ->and($body['data'][0]['action'])->toBe('auth.login');
});
