<?php

declare(strict_types=1);

use App\Modules\Health\HealthController;

it('reports ok status and a reachable database', function () {
    $response = (new HealthController(test_connection()))->index();
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['status'])->toBe('ok')
        ->and($body['database'])->toBe('ok')
        ->and($body)->toHaveKey('time');
});
