<?php

declare(strict_types=1);

namespace App\Modules\Clients;

use App\Database\Connection;
use App\Modules\Auth\Middleware\AuthenticateMiddleware;
use App\Modules\Clients\Dto\CreateClientDto;
use App\Modules\Clients\Dto\UpdateClientDto;
use App\Support\ActivityLogger;
use Hyperdrive\Attributes\Http\Route;
use Hyperdrive\Attributes\Http\Verbs\Delete;
use Hyperdrive\Attributes\Http\Verbs\Get;
use Hyperdrive\Attributes\Http\Verbs\Patch;
use Hyperdrive\Attributes\Http\Verbs\Post;
use Hyperdrive\Attributes\Middleware;
use Hyperdrive\Http\Request;
use Hyperdrive\Http\Response;

// Backs the frontend's Clients page: a paginated, searchable list plus
// view/create/update/delete. Every field here is named to match the
// frontend's Client type exactly (company/contact/email/status/projects)
// even though the clients table itself uses company_name/contact_name -
// present()/the DTOs are the only place that mapping happens.
#[Route('/clients')]
#[Middleware([AuthenticateMiddleware::class])]
class ClientsController
{
    private const STATUSES = ['active', 'prospect', 'inactive'];

    public function __construct(
        private readonly Connection $connection,
        private readonly ActivityLogger $activityLogger,
    ) {
    }

    #[Get]
    public function index(Request $request): Response
    {
        $page = max(1, (int) $request->query('page', 1));
        $pageSize = max(1, min(100, (int) $request->query('pageSize', 10)));
        $search = trim((string) $request->query('q', ''));

        $pdo = $this->connection->pdo();

        $where = '';
        $params = [];
        if ($search !== '') {
            $where = 'WHERE c.company_name ILIKE :search OR c.contact_name ILIKE :search OR c.email ILIKE :search';
            $params['search'] = "%{$search}%";
        }

        $countStatement = $pdo->prepare("SELECT COUNT(*) FROM clients c {$where}");
        $countStatement->execute($params);
        $total = (int) $countStatement->fetchColumn();

        $offset = ($page - 1) * $pageSize;

        // LIMIT/OFFSET interpolated directly (not bound) - both are
        // already cast to int above, so there's nothing to inject, and
        // Postgres via PDO can be finicky binding these as typed integers.
        $statement = $pdo->prepare(<<<SQL
            SELECT c.id, c.company_name, c.contact_name, c.email, c.status,
                   COUNT(p.id) AS projects_count
            FROM clients c
            LEFT JOIN projects p ON p.client_id = c.id
            {$where}
            GROUP BY c.id
            ORDER BY c.created_at DESC
            LIMIT {$pageSize} OFFSET {$offset}
        SQL);
        $statement->execute($params);

        return Response::json([
            'data' => array_map($this->present(...), $statement->fetchAll()),
            'page' => $page,
            'pageSize' => $pageSize,
            'total' => $total,
        ]);
    }

    #[Get('/{id}')]
    public function show(string $id): Response
    {
        $client = $this->find($id);

        if ($client === null) {
            return Response::json(['error' => 'Client not found'], 404);
        }

        return Response::json($this->present($client));
    }

    #[Post]
    public function store(CreateClientDto $dto, Request $request): Response
    {
        if (!in_array($dto->status, self::STATUSES, true)) {
            return Response::json(['error' => 'status must be one of: active, prospect, inactive'], 422);
        }

        $statement = $this->connection->pdo()->prepare(<<<SQL
            INSERT INTO clients (company_name, contact_name, email, status)
            VALUES (:company_name, :contact_name, :email, :status)
            RETURNING id, company_name, contact_name, email, status
        SQL);
        $statement->execute([
            'company_name' => $dto->company,
            'contact_name' => $dto->contact,
            'email' => $dto->email,
            'status' => $dto->status,
        ]);
        $client = $statement->fetch();
        $client['projects_count'] = 0;

        $this->activityLogger->log(
            action: 'client.created',
            description: "Created client {$dto->company}",
            userId: $request->getAttribute('auth_user')['sub'] ?? null,
            subjectType: 'client',
            subjectId: $client['id'],
        );

        return Response::json($this->present($client), 201);
    }

    #[Patch('/{id}')]
    public function update(string $id, UpdateClientDto $dto, Request $request): Response
    {
        if ($this->find($id) === null) {
            return Response::json(['error' => 'Client not found'], 404);
        }

        if (!in_array($dto->status, self::STATUSES, true)) {
            return Response::json(['error' => 'status must be one of: active, prospect, inactive'], 422);
        }

        $statement = $this->connection->pdo()->prepare(<<<SQL
            UPDATE clients
            SET company_name = :company_name, contact_name = :contact_name,
                email = :email, status = :status, updated_at = NOW()
            WHERE id = :id
        SQL);
        $statement->execute([
            'id' => $id,
            'company_name' => $dto->company,
            'contact_name' => $dto->contact,
            'email' => $dto->email,
            'status' => $dto->status,
        ]);

        $this->activityLogger->log(
            action: 'client.updated',
            description: "Updated client {$dto->company}",
            userId: $request->getAttribute('auth_user')['sub'] ?? null,
            subjectType: 'client',
            subjectId: $id,
        );

        return Response::json($this->present($this->find($id)));
    }

    #[Delete('/{id}')]
    public function destroy(string $id, Request $request): Response
    {
        $client = $this->find($id);
        if ($client === null) {
            return Response::json(['error' => 'Client not found'], 404);
        }

        $statement = $this->connection->pdo()->prepare('DELETE FROM clients WHERE id = :id');
        $statement->execute(['id' => $id]);

        $this->activityLogger->log(
            action: 'client.deleted',
            description: "Deleted client {$client['company_name']}",
            userId: $request->getAttribute('auth_user')['sub'] ?? null,
            subjectType: 'client',
            subjectId: $id,
        );

        return Response::json(['message' => 'Client deleted']);
    }

    /**
     * @return array<string, mixed>|null
     */
    private function find(string $id): ?array
    {
        $statement = $this->connection->pdo()->prepare(<<<SQL
            SELECT c.id, c.company_name, c.contact_name, c.email, c.status,
                   COUNT(p.id) AS projects_count
            FROM clients c
            LEFT JOIN projects p ON p.client_id = c.id
            WHERE c.id = :id
            GROUP BY c.id
        SQL);
        $statement->execute(['id' => $id]);
        $row = $statement->fetch();

        return $row === false ? null : $row;
    }

    /**
     * @param array<string, mixed> $row
     * @return array<string, mixed>
     */
    private function present(array $row): array
    {
        return [
            'id' => $row['id'],
            'company' => $row['company_name'],
            'contact' => $row['contact_name'],
            'email' => $row['email'],
            'status' => $row['status'],
            'projects' => (int) $row['projects_count'],
        ];
    }
}
