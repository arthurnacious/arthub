<?php

declare(strict_types=1);

namespace App\Events;

// Dispatched specifically when an invoice's status transitions to 'paid'
// (not on every update) - separate from InvoiceUpdated so future features
// (e.g. a "you got paid" notification, once online payments/notifications
// exist) can listen for exactly this without filtering every update.
final class InvoicePaid
{
    public function __construct(
        public readonly string $invoiceId,
        public readonly string $invoiceNumber,
        public readonly float $total,
        public readonly ?string $userId,
    ) {
    }
}
