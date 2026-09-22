<?php

declare(strict_types=1);

namespace App\Modules\Activity;

use App\Database\Connection;
use App\Modules\Auth\Middleware\AuthenticateMiddleware;
use Hyperdrive\Attributes\Http\Route;
use Hyperdrive\Attributes\Http\Verbs\Get;
use Hyperdrive\Attributes\Middleware;
use Hyperdrive\Http\Response;

// Backs the frontend's terminal-style activity log panel.
#[Route('/activity-log')]
#[Middleware([AuthenticateMiddleware::class])]
class ActivityController
{
    public function __construct(private readonly Connection $connection)
    {
    }

    #[Get]
    public function index(): Response
    {
        $statement = $this->connection->pdo()->query(<<<SQL
            SELECT id, user_id, action, description, subject_type, subject_id, metadata, created_at
            FROM activity_log
            ORDER BY created_at DESC
            LIMIT 100
        SQL);

        return Response::json(['data' => $statement->fetchAll()]);
    }
}
