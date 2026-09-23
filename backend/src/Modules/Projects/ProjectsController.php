<?php

declare(strict_types=1);

namespace App\Modules\Projects;

use App\Database\Connection;
use App\Modules\Auth\Middleware\AuthenticateMiddleware;
use App\Modules\Projects\Dto\CreateProjectDto;
use App\Modules\Projects\Dto\UpdateProjectDto;
use App\Support\ActivityLogger;
use Hyperdrive\Attributes\Http\Route;
use Hyperdrive\Attributes\Http\Verbs\Delete;
use Hyperdrive\Attributes\Http\Verbs\Get;
use Hyperdrive\Attributes\Http\Verbs\Patch;
use Hyperdrive\Attributes\Http\Verbs\Post;
use Hyperdrive\Attributes\Middleware;
use Hyperdrive\Http\Request;
use Hyperdrive\Http\Response;

// Portfolio-style project list: cards with a cover image (or an ASCII
// fallback on the frontend when there isn't one), categorized, each
// belonging to a client. Every field is named to match the frontend's
// Project type exactly, even though the table itself uses snake_case -
// present()/the DTOs are the only place that mapping happens.
#[Route('/projects')]
#[Middleware([AuthenticateMiddleware::class])]
class ProjectsController
{
    private const STATUSES = ['active', 'completed', 'archived'];
    private const MAX_IMAGE_BYTES = 5 * 1024 * 1024; // 5MB
    private const ALLOWED_IMAGE_MIME_TYPES = ['image/jpeg', 'image/png', 'image/webp', 'image/gif'];

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
        $category = trim((string) $request->query('category', ''));

        $pdo = $this->connection->pdo();

        $conditions = [];
        $params = [];
        if ($search !== '') {
            $conditions[] = '(p.name ILIKE :search OR p.description ILIKE :search)';
            $params['search'] = "%{$search}%";
        }
        if ($category !== '') {
            $conditions[] = 'p.category = :category';
            $params['category'] = $category;
        }
        $where = $conditions === [] ? '' : 'WHERE ' . implode(' AND ', $conditions);

        $countStatement = $pdo->prepare("SELECT COUNT(*) FROM projects p {$where}");
        $countStatement->execute($params);
        $total = (int) $countStatement->fetchColumn();

        $offset = ($page - 1) * $pageSize;

        // Order by id as well as created_at (see ActivityController/
        // ClientsController for why - NOW() is fixed for the whole
        // transaction, so it alone can't break ties deterministically).
        //
        // LIMIT/OFFSET interpolated directly (not bound) - both are
        // already cast to int above, so there's nothing to inject.
        $statement = $pdo->prepare(<<<SQL
            SELECT p.id, p.client_id, c.company_name AS client_name, p.name,
                   p.description, p.category, p.url, p.status,
                   (
                       SELECT pi.path FROM project_images pi
                       WHERE pi.project_id = p.id
                       ORDER BY pi.sort_order ASC, pi.created_at ASC
                       LIMIT 1
                   ) AS cover_image
            FROM projects p
            JOIN clients c ON c.id = p.client_id
            {$where}
            ORDER BY p.created_at DESC, p.id DESC
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

    #[Get('/categories')]
    public function categories(): Response
    {
        $statement = $this->connection->pdo()->query(
            'SELECT DISTINCT category FROM projects ORDER BY category ASC',
        );

        return Response::json(['data' => array_column($statement->fetchAll(), 'category')]);
    }

    #[Get('/{id}')]
    public function show(string $id): Response
    {
        $project = $this->find($id);

        if ($project === null) {
            return Response::json(['error' => 'Project not found'], 404);
        }

        return Response::json($this->presentDetail($project));
    }

    #[Post]
    public function store(CreateProjectDto $dto, Request $request): Response
    {
        if (!in_array($dto->status, self::STATUSES, true)) {
            return Response::json(['error' => 'status must be one of: active, completed, archived'], 422);
        }

        $pdo = $this->connection->pdo();

        $clientExists = $pdo->prepare('SELECT 1 FROM clients WHERE id = :id');
        $clientExists->execute(['id' => $dto->clientId]);
        if ($clientExists->fetchColumn() === false) {
            return Response::json(['error' => 'No client with that id'], 422);
        }

        $statement = $pdo->prepare(<<<SQL
            INSERT INTO projects (client_id, name, description, category, url, status)
            VALUES (:client_id, :name, :description, :category, :url, :status)
            RETURNING id
        SQL);
        $statement->execute([
            'client_id' => $dto->clientId,
            'name' => $dto->name,
            'description' => $dto->description,
            'category' => $dto->category,
            'url' => $dto->url,
            'status' => $dto->status,
        ]);
        $id = $statement->fetchColumn();

        $this->activityLogger->log(
            action: 'project.created',
            description: "Created project {$dto->name}",
            userId: $request->getAttribute('auth_user')['sub'] ?? null,
            subjectType: 'project',
            subjectId: $id,
        );

        return Response::json($this->presentDetail($this->find($id)), 201);
    }

    #[Patch('/{id}')]
    public function update(string $id, UpdateProjectDto $dto, Request $request): Response
    {
        if ($this->find($id) === null) {
            return Response::json(['error' => 'Project not found'], 404);
        }

        if (!in_array($dto->status, self::STATUSES, true)) {
            return Response::json(['error' => 'status must be one of: active, completed, archived'], 422);
        }

        $pdo = $this->connection->pdo();

        $clientExists = $pdo->prepare('SELECT 1 FROM clients WHERE id = :id');
        $clientExists->execute(['id' => $dto->clientId]);
        if ($clientExists->fetchColumn() === false) {
            return Response::json(['error' => 'No client with that id'], 422);
        }

        $statement = $pdo->prepare(<<<SQL
            UPDATE projects
            SET client_id = :client_id, name = :name, description = :description,
                category = :category, url = :url, status = :status, updated_at = NOW()
            WHERE id = :id
        SQL);
        $statement->execute([
            'id' => $id,
            'client_id' => $dto->clientId,
            'name' => $dto->name,
            'description' => $dto->description,
            'category' => $dto->category,
            'url' => $dto->url,
            'status' => $dto->status,
        ]);

        $this->activityLogger->log(
            action: 'project.updated',
            description: "Updated project {$dto->name}",
            userId: $request->getAttribute('auth_user')['sub'] ?? null,
            subjectType: 'project',
            subjectId: $id,
        );

        return Response::json($this->presentDetail($this->find($id)));
    }

    #[Delete('/{id}')]
    public function destroy(string $id, Request $request): Response
    {
        $project = $this->find($id);
        if ($project === null) {
            return Response::json(['error' => 'Project not found'], 404);
        }

        $pdo = $this->connection->pdo();

        $images = $pdo->prepare('SELECT path FROM project_images WHERE project_id = :id');
        $images->execute(['id' => $id]);
        foreach ($images->fetchAll() as $image) {
            $this->deleteImageFile($image['path']);
        }

        // project_images rows cascade-delete via the FK.
        $pdo->prepare('DELETE FROM projects WHERE id = :id')->execute(['id' => $id]);

        $this->activityLogger->log(
            action: 'project.deleted',
            description: "Deleted project {$project['name']}",
            userId: $request->getAttribute('auth_user')['sub'] ?? null,
            subjectType: 'project',
            subjectId: $id,
        );

        return Response::json(['message' => 'Project deleted']);
    }

    #[Post('/{id}/images')]
    public function uploadImages(string $id, Request $request): Response
    {
        if ($this->find($id) === null) {
            return Response::json(['error' => 'Project not found'], 404);
        }

        $files = $request->file('images');
        if ($files === null || $files === []) {
            return Response::json(['error' => 'No images given (field name: images)'], 422);
        }

        $pdo = $this->connection->pdo();
        $countStatement = $pdo->prepare('SELECT COUNT(*) FROM project_images WHERE project_id = :id');
        $countStatement->execute(['id' => $id]);
        $nextOrder = (int) $countStatement->fetchColumn();

        $insert = $pdo->prepare(<<<SQL
            INSERT INTO project_images (project_id, path, sort_order)
            VALUES (:project_id, :path, :sort_order)
            RETURNING id, path, sort_order
        SQL);

        $created = [];
        foreach ($files as $file) {
            if (!$file->isValid()) {
                return Response::json(['error' => 'One of the uploaded files was invalid'], 422);
            }
            if (!in_array($file->getClientMimeType(), self::ALLOWED_IMAGE_MIME_TYPES, true)) {
                return Response::json(['error' => 'Images must be JPEG, PNG, WEBP or GIF'], 422);
            }
            if ($file->getSize() > self::MAX_IMAGE_BYTES) {
                return Response::json(['error' => 'Each image must be 5MB or smaller'], 422);
            }

            $filename = bin2hex(random_bytes(16)) . '.' . strtolower($file->getClientOriginalExtension());
            $file->move(getcwd() . "/storage/app/projects/{$id}", $filename);
            $path = "projects/{$id}/{$filename}";

            $insert->execute(['project_id' => $id, 'path' => $path, 'sort_order' => $nextOrder]);
            $created[] = $this->presentImage($insert->fetch());
            $nextOrder++;
        }

        return Response::json(['data' => $created], 201);
    }

    #[Delete('/{id}/images/{imageId}')]
    public function deleteImage(string $id, string $imageId): Response
    {
        $statement = $this->connection->pdo()->prepare(
            'SELECT path FROM project_images WHERE id = :id AND project_id = :project_id',
        );
        $statement->execute(['id' => $imageId, 'project_id' => $id]);
        $image = $statement->fetch();

        if ($image === false) {
            return Response::json(['error' => 'Image not found'], 404);
        }

        $this->deleteImageFile($image['path']);

        $this->connection->pdo()
            ->prepare('DELETE FROM project_images WHERE id = :id')
            ->execute(['id' => $imageId]);

        return Response::json(['message' => 'Image deleted']);
    }

    /**
     * @return array<string, mixed>|null
     */
    private function find(string $id): ?array
    {
        $statement = $this->connection->pdo()->prepare(<<<SQL
            SELECT p.id, p.client_id, c.company_name AS client_name, p.name,
                   p.description, p.category, p.url, p.status
            FROM projects p
            JOIN clients c ON c.id = p.client_id
            WHERE p.id = :id
        SQL);
        $statement->execute(['id' => $id]);
        $row = $statement->fetch();

        return $row === false ? null : $row;
    }

    private function deleteImageFile(string $path): void
    {
        $fullPath = getcwd() . "/storage/app/{$path}";
        if (is_file($fullPath)) {
            unlink($fullPath);
        }
    }

    /**
     * @param array<string, mixed> $row
     * @return array<string, mixed>
     */
    private function present(array $row): array
    {
        return [
            'id' => $row['id'],
            'clientId' => $row['client_id'],
            'clientName' => $row['client_name'],
            'name' => $row['name'],
            'description' => $row['description'],
            'category' => $row['category'],
            'url' => $row['url'],
            'status' => $row['status'],
            'coverImage' => isset($row['cover_image']) && $row['cover_image'] !== null
                ? $this->imageUrl($row['cover_image'])
                : null,
        ];
    }

    /**
     * @param array<string, mixed> $row
     * @return array<string, mixed>
     */
    private function presentDetail(array $row): array
    {
        $statement = $this->connection->pdo()->prepare(<<<SQL
            SELECT id, path, sort_order
            FROM project_images
            WHERE project_id = :id
            ORDER BY sort_order ASC, created_at ASC
        SQL);
        $statement->execute(['id' => $row['id']]);

        return [
            'id' => $row['id'],
            'clientId' => $row['client_id'],
            'clientName' => $row['client_name'],
            'name' => $row['name'],
            'description' => $row['description'],
            'category' => $row['category'],
            'url' => $row['url'],
            'status' => $row['status'],
            'images' => array_map($this->presentImage(...), $statement->fetchAll()),
        ];
    }

    /**
     * @param array<string, mixed> $row
     * @return array<string, mixed>
     */
    private function presentImage(array $row): array
    {
        return [
            'id' => $row['id'],
            'url' => $this->imageUrl($row['path']),
            'sortOrder' => (int) $row['sort_order'],
        ];
    }

    private function imageUrl(string $path): string
    {
        $appUrl = rtrim($_ENV['APP_URL'] ?? '', '/');
        return "{$appUrl}/storage/{$path}";
    }
}
