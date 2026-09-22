<?php

declare(strict_types=1);

namespace App\Modules\Activity;

use App\Database\Connection;
use App\Modules\Auth\Middleware\AuthenticateMiddleware;
use Hyperdrive\Attributes\Http\Route;
use Hyperdrive\Attributes\Http\Verbs\Get;
use Hyperdrive\Attributes\Middleware;
use Hyperdrive\Http\Request;
use Hyperdrive\Http\Response;

// Backs the frontend's notifications dropdown (unpaginated: page/pageSize
// left at their defaults) and the dashboard's paginated/searchable audit
// log panel (explicit page/pageSize/q) - same endpoint either way.
#[Route('/activity-log')]
#[Middleware([AuthenticateMiddleware::class])]
class ActivityController
{
    public function __construct(private readonly Connection $connection)
    {
    }

    #[Get]
    public function index(Request $request): Response
    {
        $page = max(1, (int) $request->query('page', 1));
        $pageSize = max(1, min(100, (int) $request->query('pageSize', 100)));
        $search = trim((string) $request->query('q', ''));

        $pdo = $this->connection->pdo();

        $where = '';
        $params = [];
        if ($search !== '') {
            $where = 'WHERE description ILIKE :search OR action ILIKE :search';
            $params['search'] = "%{$search}%";
        }

        $countStatement = $pdo->prepare("SELECT COUNT(*) FROM activity_log {$where}");
        $countStatement->execute($params);
        $total = (int) $countStatement->fetchColumn();

        $offset = ($page - 1) * $pageSize;

        // Order by id as well as created_at: Postgres's NOW() returns the
        // enclosing transaction's start time, so multiple log() calls in
        // the same request/transaction can share an identical created_at,
        // making created_at alone a non-deterministic sort key.
        //
        // LIMIT/OFFSET interpolated directly (not bound) - both are
        // already cast to int above, so there's nothing to inject, and
        // Postgres via PDO can be finicky binding these as typed integers.
        $statement = $pdo->prepare(<<<SQL
            SELECT id, user_id, action, description, subject_type, subject_id, metadata, created_at
            FROM activity_log
            {$where}
            ORDER BY created_at DESC, id DESC
            LIMIT {$pageSize} OFFSET {$offset}
        SQL);
        $statement->execute($params);

        return Response::json([
            'data' => $statement->fetchAll(),
            'page' => $page,
            'pageSize' => $pageSize,
            'total' => $total,
        ]);
    }
}
