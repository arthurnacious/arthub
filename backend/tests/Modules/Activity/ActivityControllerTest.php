<?php

declare(strict_types=1);

use App\Modules\Activity\ActivityController;
use App\Support\ActivityLogger;

it('lists recent activity, most recent first', function () {
    $logger = new ActivityLogger(test_connection());
    $logger->log('client.created', 'First event');
    $logger->log('client.updated', 'Second event');

    $response = (new ActivityController(test_connection()))->index();
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['data'][0]['description'])->toBe('Second event')
        ->and($body['data'][1]['description'])->toBe('First event');
});

it('returns an empty list when nothing has happened yet', function () {
    $response = (new ActivityController(test_connection()))->index();
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['data'])->toBe([]);
});
