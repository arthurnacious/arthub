<?php

declare(strict_types=1);

namespace App\Events;

final class ClientDeleted
{
    public function __construct(
        public readonly string $clientId,
        public readonly string $company,
        public readonly ?string $userId,
    ) {
    }
}
