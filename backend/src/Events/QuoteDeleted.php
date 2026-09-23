<?php

declare(strict_types=1);

namespace App\Events;

final class QuoteDeleted
{
    public function __construct(
        public readonly string $quoteId,
        public readonly string $quoteNumber,
        public readonly ?string $userId,
    ) {
    }
}
