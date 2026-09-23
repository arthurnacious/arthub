<?php

declare(strict_types=1);

use App\Modules\Quotes\Dto\CreateQuoteDto;
use App\Modules\Quotes\Dto\UpdateQuoteDto;
use App\Modules\Quotes\QuotesController;
use Hyperdrive\Http\Request;

function make_quotes_controller(): QuotesController
{
    return new QuotesController(test_connection(), make_event_dispatcher());
}

it('lists quotes newest first, with pagination metadata', function () {
    create_test_quote();
    create_test_quote();

    $response = make_quotes_controller()->index(new Request());
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['data'])->toHaveCount(2)
        ->and($body['total'])->toBe(2)
        ->and($body['page'])->toBe(1)
        ->and($body['pageSize'])->toBe(10);
});

it('paginates', function () {
    create_test_quote();
    create_test_quote();
    create_test_quote();

    $response = make_quotes_controller()->index(new Request(query: ['page' => '2', 'pageSize' => '2']));
    $body = json_decode($response->getContent(), true);

    expect($body['data'])->toHaveCount(1)
        ->and($body['total'])->toBe(3)
        ->and($body['page'])->toBe(2);
});

it('searches by quote number, project name or client name', function () {
    $clientId = create_test_client(company: 'Nimbus Retail', email: 'quote-search@test.com');
    $projectId = create_test_project(clientId: $clientId, name: 'Nimbus Website');
    create_test_quote(projectId: $projectId);
    create_test_quote();

    $response = make_quotes_controller()->index(new Request(query: ['q' => 'nimbus']));
    $body = json_decode($response->getContent(), true);

    expect($body['data'])->toHaveCount(1)
        ->and($body['data'][0]['projectName'])->toBe('Nimbus Website');
});

it('filters by status', function () {
    create_test_quote(status: 'draft');
    create_test_quote(status: 'sent');

    $response = make_quotes_controller()->index(new Request(query: ['status' => 'sent']));
    $body = json_decode($response->getContent(), true);

    expect($body['data'])->toHaveCount(1)
        ->and($body['data'][0]['status'])->toBe('sent');
});

it('shows a single quote with its client/project names and empty lines', function () {
    $clientId = create_test_client(company: 'Solo Co', email: 'solo-quote@test.com');
    $projectId = create_test_project(clientId: $clientId, name: 'Solo Project');
    $id = create_test_quote(projectId: $projectId);

    $response = make_quotes_controller()->show($id);
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['projectName'])->toBe('Solo Project')
        ->and($body['clientName'])->toBe('Solo Co')
        ->and($body['lines'])->toBe([])
        ->and((float) $body['subtotal'])->toBe(0.0)
        ->and((float) $body['total'])->toBe(0.0);
});

it('404s for an unknown quote id', function () {
    $response = make_quotes_controller()->show('00000000-0000-0000-0000-000000000000');

    expect($response->getStatusCode())->toBe(404);
});

it('creates a quote with line items, generates a sequential number, and logs the activity', function () {
    $userId = create_test_user(email: 'quote-creator@test.com');
    $projectId = create_test_project();
    $dto = new CreateQuoteDto([
        'projectId' => $projectId,
        'status' => 'draft',
        'notes' => 'Thanks for the opportunity',
    ]);

    $response = make_quotes_controller()->store(
        $dto,
        new Request(
            attributes: ['auth_user' => ['sub' => $userId]],
            content: json_encode([
                'lineItems' => [
                    ['description' => 'Design', 'quantity' => 2, 'unitPrice' => 100],
                    ['description' => 'Development', 'quantity' => 10, 'unitPrice' => 80],
                ],
            ]),
            server: ['CONTENT_TYPE' => 'application/json'],
        ),
    );
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(201)
        ->and($body['quoteNumber'])->toMatch('/^Q-\d{4,}$/')
        ->and($body['status'])->toBe('draft')
        ->and($body['lines'])->toHaveCount(2)
        ->and((float) $body['subtotal'])->toBe(1000.0)
        ->and((float) $body['total'])->toBe(1000.0);

    $log = test_connection()->pdo()
        ->query("SELECT * FROM activity_log WHERE action = 'quote.created'")
        ->fetch();
    expect($log['description'])->toBe("Created quote {$body['quoteNumber']}")
        ->and($log['user_id'])->toBe($userId);
});

it('generates unique, increasing quote numbers across creates', function () {
    $projectId = create_test_project();

    $first = make_quotes_controller()->store(
        new CreateQuoteDto(['projectId' => $projectId, 'status' => 'draft']),
        new Request(),
    );
    $second = make_quotes_controller()->store(
        new CreateQuoteDto(['projectId' => $projectId, 'status' => 'draft']),
        new Request(),
    );

    $firstNumber = json_decode($first->getContent(), true)['quoteNumber'];
    $secondNumber = json_decode($second->getContent(), true)['quoteNumber'];

    expect($firstNumber)->not->toBe($secondNumber);
});

it('computes each line total with its own discount, then the quote-level discount on top', function () {
    $projectId = create_test_project();
    $dto = new CreateQuoteDto(['projectId' => $projectId, 'status' => 'draft', 'discountType' => 'fixed', 'discountValue' => 50]);

    $response = make_quotes_controller()->store(
        $dto,
        new Request(content: json_encode([
            'lineItems' => [
                // 2 * 100 = 200, minus 10% = 180
                ['description' => 'A', 'quantity' => 2, 'unitPrice' => 100, 'discountType' => 'percent', 'discountValue' => 10],
                // 1 * 50 = 50, minus $5 fixed = 45
                ['description' => 'B', 'quantity' => 1, 'unitPrice' => 50, 'discountType' => 'fixed', 'discountValue' => 5],
            ],
        ]), server: ['CONTENT_TYPE' => 'application/json']),
    );
    $body = json_decode($response->getContent(), true);

    expect((float) $body['lines'][0]['lineTotal'])->toBe(180.0)
        ->and((float) $body['lines'][1]['lineTotal'])->toBe(45.0)
        // subtotal = 180 + 45 = 225, quote-level discount $50 -> total 175
        ->and((float) $body['subtotal'])->toBe(225.0)
        ->and((float) $body['discount'])->toBe(50.0)
        ->and((float) $body['total'])->toBe(175.0);
});

it('rejects an invalid status on create', function () {
    $projectId = create_test_project();
    $dto = new CreateQuoteDto(['projectId' => $projectId, 'status' => 'bogus']);

    $response = make_quotes_controller()->store($dto, new Request());

    expect($response->getStatusCode())->toBe(422);
});

it('rejects an invalid discountType on create', function () {
    $projectId = create_test_project();
    $dto = new CreateQuoteDto(['projectId' => $projectId, 'status' => 'draft', 'discountType' => 'bogus']);

    $response = make_quotes_controller()->store($dto, new Request());

    expect($response->getStatusCode())->toBe(422);
});

it('rejects creating a quote for an unknown project', function () {
    $dto = new CreateQuoteDto(['projectId' => '00000000-0000-0000-0000-000000000000', 'status' => 'draft']);

    $response = make_quotes_controller()->store($dto, new Request());

    expect($response->getStatusCode())->toBe(422);
});

it('rejects a line item with no description', function () {
    $projectId = create_test_project();
    $dto = new CreateQuoteDto(['projectId' => $projectId, 'status' => 'draft']);

    $response = make_quotes_controller()->store($dto, new Request(content: json_encode([
        'lineItems' => [['description' => '', 'quantity' => 1, 'unitPrice' => 10]],
    ]), server: ['CONTENT_TYPE' => 'application/json']));

    expect($response->getStatusCode())->toBe(422);
});

it('rejects a line item with a non-positive quantity', function () {
    $projectId = create_test_project();
    $dto = new CreateQuoteDto(['projectId' => $projectId, 'status' => 'draft']);

    $response = make_quotes_controller()->store($dto, new Request(content: json_encode([
        'lineItems' => [['description' => 'A', 'quantity' => 0, 'unitPrice' => 10]],
    ]), server: ['CONTENT_TYPE' => 'application/json']));

    expect($response->getStatusCode())->toBe(422);
});

it('rejects a line item with a negative unit price', function () {
    $projectId = create_test_project();
    $dto = new CreateQuoteDto(['projectId' => $projectId, 'status' => 'draft']);

    $response = make_quotes_controller()->store($dto, new Request(content: json_encode([
        'lineItems' => [['description' => 'A', 'quantity' => 1, 'unitPrice' => -1]],
    ]), server: ['CONTENT_TYPE' => 'application/json']));

    expect($response->getStatusCode())->toBe(422);
});

it('rejects a line item with an invalid discountType', function () {
    $projectId = create_test_project();
    $dto = new CreateQuoteDto(['projectId' => $projectId, 'status' => 'draft']);

    $response = make_quotes_controller()->store($dto, new Request(content: json_encode([
        'lineItems' => [['description' => 'A', 'quantity' => 1, 'unitPrice' => 10, 'discountType' => 'bogus']],
    ]), server: ['CONTENT_TYPE' => 'application/json']));

    expect($response->getStatusCode())->toBe(422);
});

it('rejects a line item with a percent discount over 100', function () {
    $projectId = create_test_project();
    $dto = new CreateQuoteDto(['projectId' => $projectId, 'status' => 'draft']);

    $response = make_quotes_controller()->store($dto, new Request(content: json_encode([
        'lineItems' => [['description' => 'A', 'quantity' => 1, 'unitPrice' => 10, 'discountType' => 'percent', 'discountValue' => 150]],
    ]), server: ['CONTENT_TYPE' => 'application/json']));

    expect($response->getStatusCode())->toBe(422);
});

it('updates a quote, replacing its line items entirely', function () {
    $id = create_test_quote();
    $pdo = test_connection()->pdo();
    $pdo->prepare('INSERT INTO quote_lines (quote_id, description, quantity, unit_price, position) VALUES (:quote_id, :description, 1, 10, 0)')
        ->execute(['quote_id' => $id, 'description' => 'Old line']);

    $projectId = create_test_project();
    $dto = new UpdateQuoteDto(['projectId' => $projectId, 'status' => 'sent']);

    $response = make_quotes_controller()->update($id, $dto, new Request(content: json_encode([
        'lineItems' => [['description' => 'New line', 'quantity' => 1, 'unitPrice' => 25]],
    ]), server: ['CONTENT_TYPE' => 'application/json']));
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['status'])->toBe('sent')
        ->and($body['projectId'])->toBe($projectId)
        ->and($body['lines'])->toHaveCount(1)
        ->and($body['lines'][0]['description'])->toBe('New line');
});

it('404s updating an unknown quote', function () {
    $projectId = create_test_project();
    $dto = new UpdateQuoteDto(['projectId' => $projectId, 'status' => 'draft']);

    $response = make_quotes_controller()->update('00000000-0000-0000-0000-000000000000', $dto, new Request());

    expect($response->getStatusCode())->toBe(404);
});

it('rejects an invalid status on update', function () {
    $id = create_test_quote();
    $projectId = create_test_project();
    $dto = new UpdateQuoteDto(['projectId' => $projectId, 'status' => 'bogus']);

    $response = make_quotes_controller()->update($id, $dto, new Request());

    expect($response->getStatusCode())->toBe(422);
});

it('deletes a quote, cascading its line items, and logs the activity', function () {
    $userId = create_test_user(email: 'quote-deleter@test.com');
    $id = create_test_quote();
    $pdo = test_connection()->pdo();
    $pdo->prepare('INSERT INTO quote_lines (quote_id, description, quantity, unit_price, position) VALUES (:quote_id, :description, 1, 10, 0)')
        ->execute(['quote_id' => $id, 'description' => 'A line']);

    $response = make_quotes_controller()->destroy($id, new Request(attributes: ['auth_user' => ['sub' => $userId]]));

    expect($response->getStatusCode())->toBe(200);

    $remainingLines = $pdo->query("SELECT COUNT(*) FROM quote_lines WHERE quote_id = '{$id}'")->fetchColumn();
    expect((int) $remainingLines)->toBe(0);

    $log = $pdo->query("SELECT * FROM activity_log WHERE action = 'quote.deleted'")->fetch();
    expect($log['user_id'])->toBe($userId);
});

it('404s deleting an unknown quote', function () {
    $response = make_quotes_controller()->destroy('00000000-0000-0000-0000-000000000000', new Request());

    expect($response->getStatusCode())->toBe(404);
});
