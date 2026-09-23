<?php

declare(strict_types=1);

namespace App\Events;

final class PasswordWasReset
{
    public function __construct(
        public readonly string $userId,
    ) {
    }
}
