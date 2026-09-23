<?php

declare(strict_types=1);

namespace App\Modules\Quotes;

use App\Database\Connection;
use App\Events\QuoteCreated;
use App\Events\QuoteDeleted;
use App\Events\QuoteUpdated;
use App\Modules\Auth\Middleware\AuthenticateMiddleware;
use App\Modules\Quotes\Dto\CreateQuoteDto;
use App\Modules\Quotes\Dto\UpdateQuoteDto;
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

// An estimate sent to a client for a project. Line items live in
// quote_lines (many per quote) and aren't part of the Dto - the frontend
// sends the whole quote, including its lines, as one JSON body, and an
// update replaces every line rather than diffing them (same "frontend
// always sends the full record" convention as everywhere else in this
// app). Both a quote-level discount and each line's own discount are
// supported (see the quotes/quote_lines migrations) - a quote's total is
// the sum of each line's own (already-discounted) total, then the
// quote-level discount applied on top of that.
#[Route('/quotes')]
#[Middleware([AuthenticateMiddleware::class])]
class QuotesController
{
    private const STATUSES = ['draft', 'sent', 'accepted', 'rejected', 'expired'];
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
            $conditions[] = '(q.quote_number ILIKE :search OR p.name ILIKE :search OR c.company_name ILIKE :search)';
            $params['search'] = "%{$search}%";
        }
        if ($status !== '') {
            $conditions[] = 'q.status = :status';
            $params['status'] = $status;
        }
        $where = $conditions === [] ? '' : 'WHERE ' . implode(' AND ', $conditions);

        $countStatement = $pdo->prepare(<<<SQL
            SELECT COUNT(*)
            FROM quotes q
            JOIN projects p ON p.id = q.project_id
            JOIN clients c ON c.id = p.client_id
            {$where}
        SQL);
        $countStatement->execute($params);
        $total = (int) $countStatement->fetchColumn();

        $offset = ($page - 1) * $pageSize;

        // Order by id as well as created_at (see ProjectsController/
        // ClientsController for why - NOW() is fixed for the whole
        // transaction, so it alone can't break ties deterministically).
        $statement = $pdo->prepare(<<<SQL
            SELECT q.id, q.quote_number, q.project_id, p.name AS project_name, p.client_id,
                   c.company_name AS client_name, q.status, q.issue_date, q.expiry_date,
                   q.discount_type, q.discount_value
            FROM quotes q
            JOIN projects p ON p.id = q.project_id
            JOIN clients c ON c.id = p.client_id
            {$where}
            ORDER BY q.created_at DESC, q.id DESC
            LIMIT {$pageSize} OFFSET {$offset}
        SQL);
        $statement->execute($params);
        $quotes = $statement->fetchAll();

        $linesByQuote = $this->fetchLinesGroupedByQuote(array_column($quotes, 'id'));

        return Response::json([
            'data' => array_map(
                fn (array $row) => $this->present($row, $linesByQuote[$row['id']] ?? []),
                $quotes,
            ),
            'page' => $page,
            'pageSize' => $pageSize,
            'total' => $total,
        ]);
    }

    #[Get('/{id}')]
    public function show(string $id): Response
    {
        $quote = $this->find($id);

        if ($quote === null) {
            return Response::json(['error' => 'Quote not found'], 404);
        }

        return Response::json($this->presentDetail($quote));
    }

    #[Post]
    public function store(CreateQuoteDto $dto, Request $request): Response
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

        $lines = $request->json('lineItems', []);
        $lineError = $this->validateLines($lines);
        if ($lineError !== null) {
            return Response::json(['error' => $lineError], 422);
        }

        $quoteNumber = $this->nextQuoteNumber($pdo);

        $statement = $pdo->prepare(<<<SQL
            INSERT INTO quotes (project_id, quote_number, status, issue_date, expiry_date, discount_type, discount_value, notes)
            VALUES (:project_id, :quote_number, :status, :issue_date, :expiry_date, :discount_type, :discount_value, :notes)
            RETURNING id
        SQL);
        $statement->execute([
            'project_id' => $dto->projectId,
            'quote_number' => $quoteNumber,
            'status' => $dto->status,
            'issue_date' => $dto->issueDate,
            'expiry_date' => $dto->expiryDate,
            'discount_type' => $dto->discountType,
            'discount_value' => $dto->discountValue,
            'notes' => $dto->notes,
        ]);
        $id = $statement->fetchColumn();

        $this->replaceLines($id, $lines);

        $this->events->dispatch(new QuoteCreated(
            quoteId: $id,
            quoteNumber: $quoteNumber,
            userId: $request->getAttribute('auth_user')['sub'] ?? null,
        ));

        return Response::json($this->presentDetail($this->find($id)), 201);
    }

    #[Patch('/{id}')]
    public function update(string $id, UpdateQuoteDto $dto, Request $request): Response
    {
        if ($this->find($id) === null) {
            return Response::json(['error' => 'Quote not found'], 404);
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

        $lines = $request->json('lineItems', []);
        $lineError = $this->validateLines($lines);
        if ($lineError !== null) {
            return Response::json(['error' => $lineError], 422);
        }

        $statement = $pdo->prepare(<<<SQL
            UPDATE quotes
            SET project_id = :project_id, status = :status, issue_date = :issue_date,
                expiry_date = :expiry_date, discount_type = :discount_type,
                discount_value = :discount_value, notes = :notes, updated_at = NOW()
            WHERE id = :id
        SQL);
        $statement->execute([
            'id' => $id,
            'project_id' => $dto->projectId,
            'status' => $dto->status,
            'issue_date' => $dto->issueDate,
            'expiry_date' => $dto->expiryDate,
            'discount_type' => $dto->discountType,
            'discount_value' => $dto->discountValue,
            'notes' => $dto->notes,
        ]);

        $this->replaceLines($id, $lines);

        $quote = $this->find($id);

        $this->events->dispatch(new QuoteUpdated(
            quoteId: $id,
            quoteNumber: $quote['quote_number'],
            userId: $request->getAttribute('auth_user')['sub'] ?? null,
        ));

        return Response::json($this->presentDetail($quote));
    }

    #[Delete('/{id}')]
    public function destroy(string $id, Request $request): Response
    {
        $quote = $this->find($id);
        if ($quote === null) {
            return Response::json(['error' => 'Quote not found'], 404);
        }

        // quote_lines cascade-delete via the FK.
        $this->connection->pdo()->prepare('DELETE FROM quotes WHERE id = :id')->execute(['id' => $id]);

        $this->events->dispatch(new QuoteDeleted(
            quoteId: $id,
            quoteNumber: $quote['quote_number'],
            userId: $request->getAttribute('auth_user')['sub'] ?? null,
        ));

        return Response::json(['message' => 'Quote deleted']);
    }

    private function nextQuoteNumber(PDO $pdo): string
    {
        $seq = (int) $pdo->query("SELECT nextval('quote_number_seq')")->fetchColumn();

        return sprintf('Q-%04d', $seq);
    }

    /**
     * Deletes every existing line for the quote and re-inserts the given
     * ones, in array order (its index becomes `position`). Simpler and
     * safer than diffing against what's already there, and matches how
     * updates work everywhere else in this app.
     *
     * @param array<int, array<string, mixed>> $lines
     */
    private function replaceLines(string $quoteId, array $lines): void
    {
        $pdo = $this->connection->pdo();
        $pdo->prepare('DELETE FROM quote_lines WHERE quote_id = :quote_id')->execute(['quote_id' => $quoteId]);

        if ($lines === []) {
            return;
        }

        $insert = $pdo->prepare(<<<SQL
            INSERT INTO quote_lines (quote_id, description, quantity, unit_price, discount_type, discount_value, position)
            VALUES (:quote_id, :description, :quantity, :unit_price, :discount_type, :discount_value, :position)
        SQL);

        foreach (array_values($lines) as $position => $line) {
            $insert->execute([
                'quote_id' => $quoteId,
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
     * @param string[] $quoteIds
     * @return array<string, array<int, array<string, mixed>>> quote id => its lines
     */
    private function fetchLinesGroupedByQuote(array $quoteIds): array
    {
        if ($quoteIds === []) {
            return [];
        }

        $placeholders = [];
        $params = [];
        foreach ($quoteIds as $i => $quoteId) {
            $placeholders[] = ":id{$i}";
            $params["id{$i}"] = $quoteId;
        }
        $inClause = implode(', ', $placeholders);

        $statement = $this->connection->pdo()->prepare(<<<SQL
            SELECT id, quote_id, description, quantity, unit_price, discount_type, discount_value, position
            FROM quote_lines
            WHERE quote_id IN ({$inClause})
            ORDER BY position ASC, created_at ASC
        SQL);
        $statement->execute($params);

        $grouped = [];
        foreach ($statement->fetchAll() as $row) {
            $grouped[$row['quote_id']][] = $row;
        }

        return $grouped;
    }

    /**
     * @return array<int, array<string, mixed>>
     */
    private function fetchLines(string $quoteId): array
    {
        $statement = $this->connection->pdo()->prepare(<<<SQL
            SELECT id, quote_id, description, quantity, unit_price, discount_type, discount_value, position
            FROM quote_lines
            WHERE quote_id = :quote_id
            ORDER BY position ASC, created_at ASC
        SQL);
        $statement->execute(['quote_id' => $quoteId]);

        return $statement->fetchAll();
    }

    /**
     * @return array<string, mixed>|null
     */
    private function find(string $id): ?array
    {
        $statement = $this->connection->pdo()->prepare(<<<SQL
            SELECT q.id, q.quote_number, q.project_id, p.name AS project_name, p.client_id,
                   c.company_name AS client_name, q.status, q.issue_date, q.expiry_date,
                   q.discount_type, q.discount_value, q.notes
            FROM quotes q
            JOIN projects p ON p.id = q.project_id
            JOIN clients c ON c.id = p.client_id
            WHERE q.id = :id
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
    private function totals(array $lines, ?string $quoteDiscountType, float $quoteDiscountValue): array
    {
        $subtotal = 0.0;
        foreach ($lines as $line) {
            $subtotal += $this->lineTotal($line);
        }

        $discount = $this->discountAmount($subtotal, $quoteDiscountType, $quoteDiscountValue);
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
            'quoteNumber' => $row['quote_number'],
            'projectId' => $row['project_id'],
            'projectName' => $row['project_name'],
            'clientId' => $row['client_id'],
            'clientName' => $row['client_name'],
            'status' => $row['status'],
            'issueDate' => $row['issue_date'],
            'expiryDate' => $row['expiry_date'],
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
            'quoteNumber' => $row['quote_number'],
            'projectId' => $row['project_id'],
            'projectName' => $row['project_name'],
            'clientId' => $row['client_id'],
            'clientName' => $row['client_name'],
            'status' => $row['status'],
            'issueDate' => $row['issue_date'],
            'expiryDate' => $row['expiry_date'],
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
