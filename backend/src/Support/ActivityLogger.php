<?php

declare(strict_types=1);

namespace App\Support;

use App\Database\Connection;

// Writes one row per interesting backend event. Read back by
// App\Modules\Activity\ActivityController for the frontend's
// terminal-style log panel.
class ActivityLogger
{
    public function __construct(private readonly Connection $connection)
    {
    }

    /**
     * @param array<string, mixed>|null $metadata extra structured detail, stored as JSONB
     */
    public function log(
        string $action,
        string $description,
        ?string $userId = null, // users.id is a UUID
        ?string $subjectType = null,
        ?string $subjectId = null, // every table's id in this schema is a UUID
        ?array $metadata = null,
    ): void {
        $statement = $this->connection->pdo()->prepare(<<<SQL
            INSERT INTO activity_log (user_id, action, description, subject_type, subject_id, metadata)
            VALUES (:user_id, :action, :description, :subject_type, :subject_id, :metadata)
        SQL);

        $statement->execute([
            'user_id' => $userId,
            'action' => $action,
            'description' => $description,
            'subject_type' => $subjectType,
            'subject_id' => $subjectId,
            'metadata' => $metadata !== null ? json_encode($metadata, JSON_THROW_ON_ERROR) : null,
        ]);
    }
}
