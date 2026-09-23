<?php

declare(strict_types=1);

namespace App\Events;

final class InvoiceDeleted
{
    public function __construct(
        public readonly string $invoiceId,
        public readonly string $invoiceNumber,
        public readonly ?string $userId,
    ) {
    }
}
