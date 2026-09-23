<?php

declare(strict_types=1);

namespace App\Modules\Invoices;

use App\Database\Connection;
use App\Events\InvoiceCreated;
use App\Events\InvoiceDeleted;
use App\Events\InvoicePaid;
use App\Events\InvoiceUpdated;
use App\Modules\Auth\Middleware\AuthenticateMiddleware;
use App\Modules\Invoices\Dto\CreateInvoiceDto;
use App\Modules\Invoices\Dto\UpdateInvoiceDto;
use Hyperdrive\Attributes\Http\Route;
use Hyperdrive\Attributes\Http\Verbs\Delete;
use Hyperdrive\Attributes\Http\Verbs\Get;
use Hyperdrive\Attributes\Http\Verbs\Patch;
use Hyperdrive\Attributes\Http\Verbs\Post;
use Hyperdrive\Attributes\Middleware;
use Hyperdrive\Events\EventDispatcher;
use Hyperdrive\Http\Request;
use Hyperdrive\Http\Response;
use PDO;

// What actually gets billed and paid. Same shape as QuotesController in
// almost every way (line items as a nested JSON array rather than a Dto
// field, an update replaces every line, both a line-level and an
// invoice-level discount) - the differences are quote_id (nullable, set
// when this invoice was generated from an accepted quote - see
// QuotesController; nothing here auto-generates one, the frontend copies
// a quote's fields/lines into a new invoice form) and paid_at, which is
// derived automatically from the status transition rather than accepted
// as input: it's set the moment status becomes 'paid' and cleared if it
// moves away again, and an InvoicePaid event (distinct from
// InvoiceUpdated) fires exactly on that transition for future features
// (e.g. a "you got paid" notification) to hook into.
#[Route('/invoices')]
#[Middleware([AuthenticateMiddleware::class])]
class InvoicesController
{
    private const STATUSES = ['draft', 'sent', 'paid', 'overdue', 'cancelled'];
    private const DISCOUNT_TYPES = ['percent', 'fixed'];

    public function __construct(
        private readonly Connection $connection,
        private readonly EventDispatcher $events,
    ) {
    }

    #[Get]
    public function index(Request $request): Response
    {
        $page = max(1, (int) $request->query('page', 1));
        $pageSize = max(1, min(100, (int) $request->query('pageSize', 10)));
        $search = trim((string) $request->query('q', ''));
        $status = trim((string) $request->query('status', ''));

        $pdo = $this->connection->pdo();

        $conditions = [];
        $params = [];
        if ($search !== '') {
            $conditions[] = '(i.invoice_number ILIKE :search OR p.name ILIKE :search OR c.company_name ILIKE :search)';
            $params['search'] = "%{$search}%";
        }
        if ($status !== '') {
            $conditions[] = 'i.status = :status';
            $params['status'] = $status;
        }
        $where = $conditions === [] ? '' : 'WHERE ' . implode(' AND ', $conditions);

        $countStatement = $pdo->prepare(<<<SQL
            SELECT COUNT(*)
            FROM invoices i
            JOIN projects p ON p.id = i.project_id
            JOIN clients c ON c.id = p.client_id
            {$where}
        SQL);
        $countStatement->execute($params);
        $total = (int) $countStatement->fetchColumn();

        $offset = ($page - 1) * $pageSize;

        $statement = $pdo->prepare(<<<SQL
            SELECT i.id, i.invoice_number, i.project_id, p.name AS project_name, p.client_id,
                   c.company_name AS client_name, i.quote_id, i.status, i.issue_date, i.due_date,
                   i.paid_at, i.discount_type, i.discount_value
            FROM invoices i
            JOIN projects p ON p.id = i.project_id
            JOIN clients c ON c.id = p.client_id
            {$where}
            ORDER BY i.created_at DESC, i.id DESC
            LIMIT {$pageSize} OFFSET {$offset}
        SQL);
        $statement->execute($params);
        $invoices = $statement->fetchAll();

        $linesByInvoice = $this->fetchLinesGroupedByInvoice(array_column($invoices, 'id'));

        return Response::json([
            'data' => array_map(
                fn (array $row) => $this->present($row, $linesByInvoice[$row['id']] ?? []),
                $invoices,
            ),
            'page' => $page,
            'pageSize' => $pageSize,
            'total' => $total,
        ]);
    }

    #[Get('/{id}')]
    public function show(string $id): Response
    {
        $invoice = $this->find($id);

        if ($invoice === null) {
            return Response::json(['error' => 'Invoice not found'], 404);
        }

        return Response::json($this->presentDetail($invoice));
    }

    #[Post]
    public function store(CreateInvoiceDto $dto, Request $request): Response
    {
        if (!in_array($dto->status, self::STATUSES, true)) {
            return Response::json(['error' => 'status must be one of: ' . implode(', ', self::STATUSES)], 422);
        }
        if ($dto->discountType !== null && !in_array($dto->discountType, self::DISCOUNT_TYPES, true)) {
            return Response::json(['error' => 'discountType must be percent, fixed, or omitted'], 422);
        }

        $pdo = $this->connection->pdo();

        $projectExists = $pdo->prepare('SELECT 1 FROM projects WHERE id = :id');
        $projectExists->execute(['id' => $dto->projectId]);
        if ($projectExists->fetchColumn() === false) {
            return Response::json(['error' => 'No project with that id'], 422);
        }

        if ($dto->quoteId !== null) {
            $quoteExists = $pdo->prepare('SELECT 1 FROM quotes WHERE id = :id');
            $quoteExists->execute(['id' => $dto->quoteId]);
            if ($quoteExists->fetchColumn() === false) {
                return Response::json(['error' => 'No quote with that id'], 422);
            }
        }

        $lines = $request->json('lineItems', []);
        $lineError = $this->validateLines($lines);
        if ($lineError !== null) {
            return Response::json(['error' => $lineError], 422);
        }

        $invoiceNumber = $this->nextInvoiceNumber($pdo);
        // A brand-new invoice's "previous status" is effectively never
        // 'paid', so paidAt(status: $dto->status, wasPaid: false) sets
        // paid_at whenever it's created directly as already paid.
        $paidAt = $this->paidAtFor($dto->status, wasPaid: false);

        $statement = $pdo->prepare(<<<SQL
            INSERT INTO invoices (project_id, quote_id, invoice_number, status, issue_date, due_date, discount_type, discount_value, paid_at, notes)
            VALUES (:project_id, :quote_id, :invoice_number, :status, :issue_date, :due_date, :discount_type, :discount_value, :paid_at, :notes)
            RETURNING id
        SQL);
        $statement->execute([
            'project_id' => $dto->projectId,
            'quote_id' => $dto->quoteId,
            'invoice_number' => $invoiceNumber,
            'status' => $dto->status,
            'issue_date' => $dto->issueDate,
            'due_date' => $dto->dueDate,
            'discount_type' => $dto->discountType,
            'discount_value' => $dto->discountValue,
            'paid_at' => $paidAt,
            'notes' => $dto->notes,
        ]);
        $id = $statement->fetchColumn();

        $this->replaceLines($id, $lines);

        $userId = $request->getAttribute('auth_user')['sub'] ?? null;
        $this->events->dispatch(new InvoiceCreated(invoiceId: $id, invoiceNumber: $invoiceNumber, userId: $userId));

        $detail = $this->presentDetail($this->find($id));
        if ($paidAt !== null) {
            $this->events->dispatch(new InvoicePaid(
                invoiceId: $id,
                invoiceNumber: $invoiceNumber,
                total: $detail['total'],
                userId: $userId,
            ));
        }

        return Response::json($detail, 201);
    }

    #[Patch('/{id}')]
    public function update(string $id, UpdateInvoiceDto $dto, Request $request): Response
    {
        $existing = $this->find($id);
        if ($existing === null) {
            return Response::json(['error' => 'Invoice not found'], 404);
        }
        if (!in_array($dto->status, self::STATUSES, true)) {
            return Response::json(['error' => 'status must be one of: ' . implode(', ', self::STATUSES)], 422);
        }
        if ($dto->discountType !== null && !in_array($dto->discountType, self::DISCOUNT_TYPES, true)) {
            return Response::json(['error' => 'discountType must be percent, fixed, or omitted'], 422);
        }

        $pdo = $this->connection->pdo();

        $projectExists = $pdo->prepare('SELECT 1 FROM projects WHERE id = :id');
        $projectExists->execute(['id' => $dto->projectId]);
        if ($projectExists->fetchColumn() === false) {
            return Response::json(['error' => 'No project with that id'], 422);
        }

        if ($dto->quoteId !== null) {
            $quoteExists = $pdo->prepare('SELECT 1 FROM quotes WHERE id = :id');
            $quoteExists->execute(['id' => $dto->quoteId]);
            if ($quoteExists->fetchColumn() === false) {
                return Response::json(['error' => 'No quote with that id'], 422);
            }
        }

        $lines = $request->json('lineItems', []);
        $lineError = $this->validateLines($lines);
        if ($lineError !== null) {
            return Response::json(['error' => $lineError], 422);
        }

        $wasPaid = $existing['status'] === 'paid';
        $paidAt = $this->paidAtFor($dto->status, $wasPaid, $existing['paid_at']);
        $becamePaid = $paidAt !== null && !$wasPaid;

        $statement = $pdo->prepare(<<<SQL
            UPDATE invoices
            SET project_id = :project_id, quote_id = :quote_id, status = :status,
                issue_date = :issue_date, due_date = :due_date, discount_type = :discount_type,
                discount_value = :discount_value, paid_at = :paid_at, notes = :notes, updated_at = NOW()
            WHERE id = :id
        SQL);
        $statement->execute([
            'id' => $id,
            'project_id' => $dto->projectId,
            'quote_id' => $dto->quoteId,
            'status' => $dto->status,
            'issue_date' => $dto->issueDate,
            'due_date' => $dto->dueDate,
            'discount_type' => $dto->discountType,
            'discount_value' => $dto->discountValue,
            'paid_at' => $paidAt,
            'notes' => $dto->notes,
        ]);

        $this->replaceLines($id, $lines);

        $invoice = $this->find($id);
        $userId = $request->getAttribute('auth_user')['sub'] ?? null;

        $this->events->dispatch(new InvoiceUpdated(
            invoiceId: $id,
            invoiceNumber: $invoice['invoice_number'],
            userId: $userId,
        ));

        $detail = $this->presentDetail($invoice);
        if ($becamePaid) {
            $this->events->dispatch(new InvoicePaid(
                invoiceId: $id,
                invoiceNumber: $invoice['invoice_number'],
                total: $detail['total'],
                userId: $userId,
            ));
        }

        return Response::json($detail);
    }

    #[Delete('/{id}')]
    public function destroy(string $id, Request $request): Response
    {
        $invoice = $this->find($id);
        if ($invoice === null) {
            return Response::json(['error' => 'Invoice not found'], 404);
        }

        // invoice_lines cascade-delete via the FK.
        $this->connection->pdo()->prepare('DELETE FROM invoices WHERE id = :id')->execute(['id' => $id]);

        $this->events->dispatch(new InvoiceDeleted(
            invoiceId: $id,
            invoiceNumber: $invoice['invoice_number'],
            userId: $request->getAttribute('auth_user')['sub'] ?? null,
        ));

        return Response::json(['message' => 'Invoice deleted']);
    }

    private function nextInvoiceNumber(PDO $pdo): string
    {
        $seq = (int) $pdo->query("SELECT nextval('invoice_number_seq')")->fetchColumn();

        return sprintf('INV-%04d', $seq);
    }

    /**
     * Derives the paid_at value for a save: set to now() the instant
     * status becomes 'paid' (unless it already was), cleared to null the
     * instant it moves away from 'paid'. $currentPaidAt is preserved when
     * status stays 'paid' across the save, rather than bumped to a new
     * timestamp every edit.
     */
    private function paidAtFor(string $status, bool $wasPaid, ?string $currentPaidAt = null): ?string
    {
        if ($status === 'paid') {
            return $wasPaid ? $currentPaidAt : (new \DateTimeImmutable())->format('Y-m-d H:i:sP');
        }

        return null;
    }

    /**
     * Deletes every existing line for the invoice and re-inserts the
     * given ones, in array order (its index becomes `position`). Same
     * "replace, don't diff" convention as QuotesController.
     *
     * @param array<int, array<string, mixed>> $lines
     */
    private function replaceLines(string $invoiceId, array $lines): void
    {
        $pdo = $this->connection->pdo();
        $pdo->prepare('DELETE FROM invoice_lines WHERE invoice_id = :invoice_id')->execute(['invoice_id' => $invoiceId]);

        if ($lines === []) {
            return;
        }

        $insert = $pdo->prepare(<<<SQL
            INSERT INTO invoice_lines (invoice_id, description, quantity, unit_price, discount_type, discount_value, position)
            VALUES (:invoice_id, :description, :quantity, :unit_price, :discount_type, :discount_value, :position)
        SQL);

        foreach (array_values($lines) as $position => $line) {
            $insert->execute([
                'invoice_id' => $invoiceId,
                'description' => $line['description'],
                'quantity' => $line['quantity'],
                'unit_price' => $line['unitPrice'],
                'discount_type' => $line['discountType'] ?? null,
                'discount_value' => $line['discountValue'] ?? 0,
                'position' => $position,
            ]);
        }
    }

    /**
     * @return string|null an error message, or null if every line is valid
     */
    private function validateLines(mixed $lines): ?string
    {
        if (!is_array($lines)) {
            return 'lineItems must be an array';
        }

        foreach ($lines as $line) {
            if (!is_array($line)) {
                return 'Each line item must be an object';
            }
            if (!isset($line['description']) || !is_string($line['description']) || trim($line['description']) === '') {
                return 'Each line item needs a description';
            }
            if (!isset($line['quantity']) || !is_numeric($line['quantity']) || (float) $line['quantity'] <= 0) {
                return 'Each line item needs a positive quantity';
            }
            if (!isset($line['unitPrice']) || !is_numeric($line['unitPrice']) || (float) $line['unitPrice'] < 0) {
                return 'Each line item needs a non-negative unit price';
            }

            $discountType = $line['discountType'] ?? null;
            if ($discountType !== null && !in_array($discountType, self::DISCOUNT_TYPES, true)) {
                return "Each line item's discountType must be percent, fixed, or omitted";
            }
            if ($discountType !== null) {
                $discountValue = $line['discountValue'] ?? null;
                if (!is_numeric($discountValue) || (float) $discountValue < 0) {
                    return 'Each line item needs a non-negative discountValue when discountType is set';
                }
                if ($discountType === 'percent' && (float) $discountValue > 100) {
                    return 'A percent discount cannot exceed 100';
                }
            }
        }

        return null;
    }

    /**
     * @param string[] $invoiceIds
     * @return array<string, array<int, array<string, mixed>>> invoice id => its lines
     */
    private function fetchLinesGroupedByInvoice(array $invoiceIds): array
    {
        if ($invoiceIds === []) {
            return [];
        }

        $placeholders = [];
        $params = [];
        foreach ($invoiceIds as $i => $invoiceId) {
            $placeholders[] = ":id{$i}";
            $params["id{$i}"] = $invoiceId;
        }
        $inClause = implode(', ', $placeholders);

        $statement = $this->connection->pdo()->prepare(<<<SQL
            SELECT id, invoice_id, description, quantity, unit_price, discount_type, discount_value, position
            FROM invoice_lines
            WHERE invoice_id IN ({$inClause})
            ORDER BY position ASC, created_at ASC
        SQL);
        $statement->execute($params);

        $grouped = [];
        foreach ($statement->fetchAll() as $row) {
            $grouped[$row['invoice_id']][] = $row;
        }

        return $grouped;
    }

    /**
     * @return array<int, array<string, mixed>>
     */
    private function fetchLines(string $invoiceId): array
    {
        $statement = $this->connection->pdo()->prepare(<<<SQL
            SELECT id, invoice_id, description, quantity, unit_price, discount_type, discount_value, position
            FROM invoice_lines
            WHERE invoice_id = :invoice_id
            ORDER BY position ASC, created_at ASC
        SQL);
        $statement->execute(['invoice_id' => $invoiceId]);

        return $statement->fetchAll();
    }

    /**
     * @return array<string, mixed>|null
     */
    private function find(string $id): ?array
    {
        $statement = $this->connection->pdo()->prepare(<<<SQL
            SELECT i.id, i.invoice_number, i.project_id, p.name AS project_name, p.client_id,
                   c.company_name AS client_name, i.quote_id, q.quote_number, i.status,
                   i.issue_date, i.due_date, i.paid_at, i.discount_type, i.discount_value, i.notes
            FROM invoices i
            JOIN projects p ON p.id = i.project_id
            JOIN clients c ON c.id = p.client_id
            LEFT JOIN quotes q ON q.id = i.quote_id
            WHERE i.id = :id
        SQL);
        $statement->execute(['id' => $id]);
        $row = $statement->fetch();

        return $row === false ? null : $row;
    }

    private function discountAmount(float $base, ?string $type, float $value): float
    {
        return match ($type) {
            'percent' => $base * ($value / 100),
            'fixed' => $value,
            default => 0.0,
        };
    }

    /**
     * @param array<string, mixed> $line
     */
    private function lineTotal(array $line): float
    {
        $subtotal = (float) $line['quantity'] * (float) $line['unit_price'];
        $discount = $this->discountAmount($subtotal, $line['discount_type'], (float) $line['discount_value']);

        return max(0.0, $subtotal - $discount);
    }

    /**
     * @param array<int, array<string, mixed>> $lines
     * @return array{subtotal: float, discount: float, total: float}
     */
    private function totals(array $lines, ?string $invoiceDiscountType, float $invoiceDiscountValue): array
    {
        $subtotal = 0.0;
        foreach ($lines as $line) {
            $subtotal += $this->lineTotal($line);
        }

        $discount = $this->discountAmount($subtotal, $invoiceDiscountType, $invoiceDiscountValue);
        $total = max(0.0, $subtotal - $discount);

        return [
            'subtotal' => round($subtotal, 2),
            'discount' => round($discount, 2),
            'total' => round($total, 2),
        ];
    }

    /**
     * @param array<string, mixed> $row
     * @param array<int, array<string, mixed>> $lines
     * @return array<string, mixed>
     */
    private function present(array $row, array $lines): array
    {
        $totals = $this->totals($lines, $row['discount_type'], (float) $row['discount_value']);

        return [
            'id' => $row['id'],
            'invoiceNumber' => $row['invoice_number'],
            'projectId' => $row['project_id'],
            'projectName' => $row['project_name'],
            'clientId' => $row['client_id'],
            'clientName' => $row['client_name'],
            'quoteId' => $row['quote_id'],
            'status' => $row['status'],
            'issueDate' => $row['issue_date'],
            'dueDate' => $row['due_date'],
            'paidAt' => $row['paid_at'],
            'total' => $totals['total'],
        ];
    }

    /**
     * @param array<string, mixed> $row
     * @return array<string, mixed>
     */
    private function presentDetail(array $row): array
    {
        $lines = $this->fetchLines($row['id']);
        $totals = $this->totals($lines, $row['discount_type'], (float) $row['discount_value']);

        return [
            'id' => $row['id'],
            'invoiceNumber' => $row['invoice_number'],
            'projectId' => $row['project_id'],
            'projectName' => $row['project_name'],
            'clientId' => $row['client_id'],
            'clientName' => $row['client_name'],
            'quoteId' => $row['quote_id'],
            'quoteNumber' => $row['quote_number'],
            'status' => $row['status'],
            'issueDate' => $row['issue_date'],
            'dueDate' => $row['due_date'],
            'paidAt' => $row['paid_at'],
            'discountType' => $row['discount_type'],
            'discountValue' => (float) $row['discount_value'],
            'notes' => $row['notes'],
            'subtotal' => $totals['subtotal'],
            'discount' => $totals['discount'],
            'total' => $totals['total'],
            'lines' => array_map($this->presentLine(...), $lines),
        ];
    }

    /**
     * @param array<string, mixed> $line
     * @return array<string, mixed>
     */
    private function presentLine(array $line): array
    {
        return [
            'id' => $line['id'],
            'description' => $line['description'],
            'quantity' => (float) $line['quantity'],
            'unitPrice' => (float) $line['unit_price'],
            'discountType' => $line['discount_type'],
            'discountValue' => (float) $line['discount_value'],
            'lineTotal' => round($this->lineTotal($line), 2),
            'position' => (int) $line['position'],
        ];
    }
}
