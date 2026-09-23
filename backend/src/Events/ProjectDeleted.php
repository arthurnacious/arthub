<?php

declare(strict_types=1);

namespace App\Events;

final class ProjectDeleted
{
    public function __construct(
        public readonly string $projectId,
        public readonly string $name,
        public readonly ?string $userId,
    ) {
    }
}
