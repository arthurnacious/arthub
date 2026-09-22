<?php

declare(strict_types=1);

namespace App\Modules\Health;

use App\Database\Connection;
use Hyperdrive\Attributes\Http\Route;
use Hyperdrive\Attributes\Http\Verbs\Get;
use Hyperdrive\Http\Response;
use PDOException;

// Liveness check - confirms the app booted AND can actually reach
// Postgres, so a broken DB connection shows up here instead of as a
// mysterious 500 on every real endpoint.
#[Route('/health')]
class HealthController
{
    public function __construct(private readonly Connection $connection)
    {
    }

    #[Get]
    public function index(): Response
    {
        try {
            $this->connection->pdo()->query('SELECT 1');
            $database = 'ok';
        } catch (PDOException) {
            $database = 'unreachable';
        }

        return Response::json([
            'status' => 'ok',
            'database' => $database,
            'time' => date(DATE_ATOM),
        ]);
    }
}
