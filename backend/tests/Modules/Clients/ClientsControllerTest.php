<?php

declare(strict_types=1);

use App\Modules\Clients\ClientsController;
use App\Modules\Clients\Dto\CreateClientDto;
use App\Modules\Clients\Dto\UpdateClientDto;
use App\Support\ActivityLogger;
use Hyperdrive\Http\Request;

function make_clients_controller(): ClientsController
{
    return new ClientsController(test_connection(), new ActivityLogger(test_connection()));
}

it('lists clients newest first, with pagination metadata', function () {
    create_test_client(company: 'Alpha Co', email: 'alpha@test.com');
    create_test_client(company: 'Beta Co', email: 'beta@test.com');

    $response = make_clients_controller()->index(new Request());
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['data'])->toHaveCount(2)
        ->and($body['data'][0]['company'])->toBe('Beta Co')
        ->and($body['total'])->toBe(2)
        ->and($body['page'])->toBe(1)
        ->and($body['pageSize'])->toBe(10);
});

it('paginates', function () {
    create_test_client(company: 'A', email: 'a@test.com');
    create_test_client(company: 'B', email: 'b@test.com');
    create_test_client(company: 'C', email: 'c@test.com');

    $response = make_clients_controller()->index(new Request(query: ['page' => '2', 'pageSize' => '2']));
    $body = json_decode($response->getContent(), true);

    expect($body['data'])->toHaveCount(1)
        ->and($body['total'])->toBe(3)
        ->and($body['page'])->toBe(2);
});

it('searches by company, contact or email', function () {
    create_test_client(company: 'Nimbus Retail', contact: 'Dana', email: 'dana@nimbus.test');
    create_test_client(company: 'Other Co', contact: 'Someone', email: 'someone@other.test');

    $response = make_clients_controller()->index(new Request(query: ['q' => 'nimbus']));
    $body = json_decode($response->getContent(), true);

    expect($body['data'])->toHaveCount(1)
        ->and($body['data'][0]['company'])->toBe('Nimbus Retail');
});

it("includes each client's project count", function () {
    $clientId = create_test_client(company: 'Has Projects', email: 'has-projects@test.com');
    $pdo = test_connection()->pdo();
    $insertProject = $pdo->prepare('INSERT INTO projects (client_id, name) VALUES (:client_id, :name)');
    $insertProject->execute(['client_id' => $clientId, 'name' => 'Project 1']);
    $insertProject->execute(['client_id' => $clientId, 'name' => 'Project 2']);

    $response = make_clients_controller()->index(new Request());
    $body = json_decode($response->getContent(), true);

    expect($body['data'][0]['projects'])->toBe(2);
});

it('shows a single client', function () {
    $id = create_test_client(company: 'Solo Co', email: 'solo@test.com');

    $response = make_clients_controller()->show($id);
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['company'])->toBe('Solo Co');
});

it('404s for an unknown client id', function () {
    $response = make_clients_controller()->show('00000000-0000-0000-0000-000000000000');

    expect($response->getStatusCode())->toBe(404);
});

it('creates a client and logs the activity', function () {
    $userId = create_test_user();
    $dto = new CreateClientDto([
        'company' => 'New Co',
        'contact' => 'Jamie Lee',
        'email' => 'jamie@newco.test',
        'status' => 'prospect',
    ]);

    $response = make_clients_controller()->store(
        $dto,
        new Request(attributes: ['auth_user' => ['sub' => $userId]]),
    );
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(201)
        ->and($body['company'])->toBe('New Co')
        ->and($body['status'])->toBe('prospect')
        ->and($body['projects'])->toBe(0);

    $log = test_connection()->pdo()
        ->query("SELECT * FROM activity_log WHERE action = 'client.created'")
        ->fetch();
    expect($log['description'])->toBe('Created client New Co')
        ->and($log['user_id'])->toBe($userId);
});

it('rejects an invalid status on create', function () {
    $dto = new CreateClientDto([
        'company' => 'New Co',
        'contact' => 'Jamie Lee',
        'email' => 'jamie@newco.test',
        'status' => 'bogus',
    ]);

    $response = make_clients_controller()->store($dto, new Request());

    expect($response->getStatusCode())->toBe(422);
});

it('updates a client', function () {
    $id = create_test_client(company: 'Old Name', email: 'old@test.com');
    $dto = new UpdateClientDto([
        'company' => 'New Name',
        'contact' => 'New Contact',
        'email' => 'new@test.com',
        'status' => 'inactive',
    ]);

    $response = make_clients_controller()->update($id, $dto, new Request());
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['company'])->toBe('New Name')
        ->and($body['status'])->toBe('inactive');
});

it('404s updating an unknown client', function () {
    $dto = new UpdateClientDto([
        'company' => 'X',
        'contact' => 'X',
        'email' => 'x@test.com',
        'status' => 'active',
    ]);

    $response = make_clients_controller()->update(
        '00000000-0000-0000-0000-000000000000',
        $dto,
        new Request(),
    );

    expect($response->getStatusCode())->toBe(404);
});

it('deletes a client', function () {
    $id = create_test_client(company: 'Gone Soon', email: 'gone@test.com');

    $response = make_clients_controller()->destroy($id, new Request());
    expect($response->getStatusCode())->toBe(200);

    $show = make_clients_controller()->show($id);
    expect($show->getStatusCode())->toBe(404);
});

it('404s deleting an unknown client', function () {
    $response = make_clients_controller()->destroy(
        '00000000-0000-0000-0000-000000000000',
        new Request(),
    );

    expect($response->getStatusCode())->toBe(404);
});
