<?php

declare(strict_types=1);

namespace App\Events;

final class UserLoggedIn
{
    public function __construct(
        public readonly string $userId,
        public readonly string $name,
    ) {
    }
}
