<?php

declare(strict_types=1);

use App\Modules\Invoices\Dto\CreateInvoiceDto;
use App\Modules\Invoices\Dto\UpdateInvoiceDto;
use App\Modules\Invoices\InvoicesController;
use Hyperdrive\Http\Request;

function make_invoices_controller(): InvoicesController
{
    return new InvoicesController(test_connection(), make_event_dispatcher());
}

it('lists invoices newest first, with pagination metadata', function () {
    create_test_invoice();
    create_test_invoice();

    $response = make_invoices_controller()->index(new Request());
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['data'])->toHaveCount(2)
        ->and($body['total'])->toBe(2)
        ->and($body['page'])->toBe(1)
        ->and($body['pageSize'])->toBe(10);
});

it('paginates', function () {
    create_test_invoice();
    create_test_invoice();
    create_test_invoice();

    $response = make_invoices_controller()->index(new Request(query: ['page' => '2', 'pageSize' => '2']));
    $body = json_decode($response->getContent(), true);

    expect($body['data'])->toHaveCount(1)
        ->and($body['total'])->toBe(3)
        ->and($body['page'])->toBe(2);
});

it('searches by invoice number, project name or client name', function () {
    $clientId = create_test_client(company: 'Nimbus Retail', email: 'invoice-search@test.com');
    $projectId = create_test_project(clientId: $clientId, name: 'Nimbus Website');
    create_test_invoice(projectId: $projectId);
    create_test_invoice();

    $response = make_invoices_controller()->index(new Request(query: ['q' => 'nimbus']));
    $body = json_decode($response->getContent(), true);

    expect($body['data'])->toHaveCount(1)
        ->and($body['data'][0]['projectName'])->toBe('Nimbus Website');
});

it('filters by status', function () {
    create_test_invoice(status: 'draft');
    create_test_invoice(status: 'sent');

    $response = make_invoices_controller()->index(new Request(query: ['status' => 'sent']));
    $body = json_decode($response->getContent(), true);

    expect($body['data'])->toHaveCount(1)
        ->and($body['data'][0]['status'])->toBe('sent');
});

it('shows a single invoice with its client/project names and empty lines', function () {
    $clientId = create_test_client(company: 'Solo Co', email: 'solo-invoice@test.com');
    $projectId = create_test_project(clientId: $clientId, name: 'Solo Project');
    $id = create_test_invoice(projectId: $projectId);

    $response = make_invoices_controller()->show($id);
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['projectName'])->toBe('Solo Project')
        ->and($body['clientName'])->toBe('Solo Co')
        ->and($body['quoteId'])->toBeNull()
        ->and($body['quoteNumber'])->toBeNull()
        ->and($body['paidAt'])->toBeNull()
        ->and($body['lines'])->toBe([]);
});

it('includes the linked quote when the invoice was generated from one', function () {
    $quoteId = create_test_quote();
    $id = create_test_invoice(quoteId: $quoteId);

    $response = make_invoices_controller()->show($id);
    $body = json_decode($response->getContent(), true);

    expect($body['quoteId'])->toBe($quoteId)
        ->and($body['quoteNumber'])->not->toBeNull();
});

it('404s for an unknown invoice id', function () {
    $response = make_invoices_controller()->show('00000000-0000-0000-0000-000000000000');

    expect($response->getStatusCode())->toBe(404);
});

it('creates an invoice with line items, generates a sequential number, and logs the activity', function () {
    $userId = create_test_user(email: 'invoice-creator@test.com');
    $projectId = create_test_project();
    $dto = new CreateInvoiceDto(['projectId' => $projectId, 'status' => 'draft']);

    $response = make_invoices_controller()->store(
        $dto,
        new Request(
            attributes: ['auth_user' => ['sub' => $userId]],
            content: json_encode(['lineItems' => [
                ['description' => 'Design', 'quantity' => 2, 'unitPrice' => 100],
            ]]),
            server: ['CONTENT_TYPE' => 'application/json'],
        ),
    );
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(201)
        ->and($body['invoiceNumber'])->toMatch('/^INV-\d{4,}$/')
        ->and($body['status'])->toBe('draft')
        ->and($body['paidAt'])->toBeNull()
        ->and((float) $body['total'])->toBe(200.0);

    $log = test_connection()->pdo()
        ->query("SELECT * FROM activity_log WHERE action = 'invoice.created'")
        ->fetch();
    expect($log['description'])->toBe("Created invoice {$body['invoiceNumber']}")
        ->and($log['user_id'])->toBe($userId);
});

it('sets paidAt and dispatches InvoicePaid when created directly as paid', function () {
    $projectId = create_test_project();
    $dto = new CreateInvoiceDto(['projectId' => $projectId, 'status' => 'paid']);

    $response = make_invoices_controller()->store($dto, new Request());
    $body = json_decode($response->getContent(), true);

    expect($body['paidAt'])->not->toBeNull();

    $log = test_connection()->pdo()
        ->query("SELECT * FROM activity_log WHERE action = 'invoice.paid'")
        ->fetch();
    expect($log['description'])->toBe("Invoice {$body['invoiceNumber']} marked paid");
});

it('rejects an invalid status on create', function () {
    $projectId = create_test_project();
    $dto = new CreateInvoiceDto(['projectId' => $projectId, 'status' => 'bogus']);

    $response = make_invoices_controller()->store($dto, new Request());

    expect($response->getStatusCode())->toBe(422);
});

it('rejects creating an invoice for an unknown project', function () {
    $dto = new CreateInvoiceDto(['projectId' => '00000000-0000-0000-0000-000000000000', 'status' => 'draft']);

    $response = make_invoices_controller()->store($dto, new Request());

    expect($response->getStatusCode())->toBe(422);
});

it('rejects creating an invoice linked to an unknown quote', function () {
    $projectId = create_test_project();
    $dto = new CreateInvoiceDto(['projectId' => $projectId, 'status' => 'draft', 'quoteId' => '00000000-0000-0000-0000-000000000000']);

    $response = make_invoices_controller()->store($dto, new Request());

    expect($response->getStatusCode())->toBe(422);
});

it('rejects a line item with no description', function () {
    $projectId = create_test_project();
    $dto = new CreateInvoiceDto(['projectId' => $projectId, 'status' => 'draft']);

    $response = make_invoices_controller()->store($dto, new Request(content: json_encode([
        'lineItems' => [['description' => '', 'quantity' => 1, 'unitPrice' => 10]],
    ]), server: ['CONTENT_TYPE' => 'application/json']));

    expect($response->getStatusCode())->toBe(422);
});

it('updates an invoice, replacing its line items entirely', function () {
    $id = create_test_invoice();
    $pdo = test_connection()->pdo();
    $pdo->prepare('INSERT INTO invoice_lines (invoice_id, description, quantity, unit_price, position) VALUES (:invoice_id, :description, 1, 10, 0)')
        ->execute(['invoice_id' => $id, 'description' => 'Old line']);

    $projectId = create_test_project();
    $dto = new UpdateInvoiceDto(['projectId' => $projectId, 'status' => 'sent']);

    $response = make_invoices_controller()->update($id, $dto, new Request(content: json_encode([
        'lineItems' => [['description' => 'New line', 'quantity' => 1, 'unitPrice' => 25]],
    ]), server: ['CONTENT_TYPE' => 'application/json']));
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['status'])->toBe('sent')
        ->and($body['projectId'])->toBe($projectId)
        ->and($body['lines'])->toHaveCount(1)
        ->and($body['lines'][0]['description'])->toBe('New line');
});

it('sets paidAt and dispatches InvoicePaid exactly once when status transitions to paid', function () {
    $id = create_test_invoice(status: 'sent');
    $projectId = test_connection()->pdo()->query("SELECT project_id FROM invoices WHERE id = '{$id}'")->fetchColumn();

    $firstUpdate = make_invoices_controller()->update(
        $id,
        new UpdateInvoiceDto(['projectId' => $projectId, 'status' => 'paid']),
        new Request(),
    );
    $firstBody = json_decode($firstUpdate->getContent(), true);
    expect($firstBody['paidAt'])->not->toBeNull();

    $paidAtAfterFirstUpdate = $firstBody['paidAt'];

    // Editing again while still 'paid' shouldn't bump paid_at to a new
    // timestamp or fire InvoicePaid a second time.
    $secondUpdate = make_invoices_controller()->update(
        $id,
        new UpdateInvoiceDto(['projectId' => $projectId, 'status' => 'paid', 'notes' => 'still paid']),
        new Request(),
    );
    $secondBody = json_decode($secondUpdate->getContent(), true);
    expect($secondBody['paidAt'])->toBe($paidAtAfterFirstUpdate);

    $paidLogCount = (int) test_connection()->pdo()
        ->query("SELECT COUNT(*) FROM activity_log WHERE action = 'invoice.paid' AND subject_id = '{$id}'")
        ->fetchColumn();
    expect($paidLogCount)->toBe(1);
});

it('clears paidAt when status moves away from paid', function () {
    $id = create_test_invoice(status: 'sent');
    $projectId = test_connection()->pdo()->query("SELECT project_id FROM invoices WHERE id = '{$id}'")->fetchColumn();

    make_invoices_controller()->update($id, new UpdateInvoiceDto(['projectId' => $projectId, 'status' => 'paid']), new Request());
    $response = make_invoices_controller()->update($id, new UpdateInvoiceDto(['projectId' => $projectId, 'status' => 'cancelled']), new Request());
    $body = json_decode($response->getContent(), true);

    expect($body['paidAt'])->toBeNull();
});

it('404s updating an unknown invoice', function () {
    $projectId = create_test_project();
    $dto = new UpdateInvoiceDto(['projectId' => $projectId, 'status' => 'draft']);

    $response = make_invoices_controller()->update('00000000-0000-0000-0000-000000000000', $dto, new Request());

    expect($response->getStatusCode())->toBe(404);
});

it('rejects an invalid status on update', function () {
    $id = create_test_invoice();
    $projectId = create_test_project();
    $dto = new UpdateInvoiceDto(['projectId' => $projectId, 'status' => 'bogus']);

    $response = make_invoices_controller()->update($id, $dto, new Request());

    expect($response->getStatusCode())->toBe(422);
});

it('deletes an invoice, cascading its line items, and logs the activity', function () {
    $userId = create_test_user(email: 'invoice-deleter@test.com');
    $id = create_test_invoice();
    $pdo = test_connection()->pdo();
    $pdo->prepare('INSERT INTO invoice_lines (invoice_id, description, quantity, unit_price, position) VALUES (:invoice_id, :description, 1, 10, 0)')
        ->execute(['invoice_id' => $id, 'description' => 'A line']);

    $response = make_invoices_controller()->destroy($id, new Request(attributes: ['auth_user' => ['sub' => $userId]]));

    expect($response->getStatusCode())->toBe(200);

    $remainingLines = $pdo->query("SELECT COUNT(*) FROM invoice_lines WHERE invoice_id = '{$id}'")->fetchColumn();
    expect((int) $remainingLines)->toBe(0);

    $log = $pdo->query("SELECT * FROM activity_log WHERE action = 'invoice.deleted'")->fetch();
    expect($log['user_id'])->toBe($userId);
});

it('404s deleting an unknown invoice', function () {
    $response = make_invoices_controller()->destroy('00000000-0000-0000-0000-000000000000', new Request());

    expect($response->getStatusCode())->toBe(404);
});
