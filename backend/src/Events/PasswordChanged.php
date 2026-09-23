<?php

declare(strict_types=1);

namespace App\Events;

// Distinct from PasswordWasReset: this is a logged-in user changing their
// own password with their current one, not the forgot-password/token flow.
final class PasswordChanged
{
    public function __construct(
        public readonly string $userId,
    ) {
    }
}
