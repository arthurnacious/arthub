<?php

declare(strict_types=1);

namespace App\Modules\Invoices\Dto;

use Hyperdrive\Http\Dto;
use Hyperdrive\Http\Dto\Validation\IsString;
use Hyperdrive\Http\Dto\Validation\NotEmpty;

// status/discountType aren't validated here as enums, and line items
// aren't part of this Dto at all - same reasoning as
// Modules\Quotes\Dto\CreateQuoteDto. `paidAt` isn't a field here either:
// InvoicesController derives it automatically from the status transition
// (set to now() the moment status becomes 'paid', cleared if it moves
// away again) rather than trusting a client-supplied timestamp.
class CreateInvoiceDto extends Dto
{
    #[IsString]
    #[NotEmpty]
    public string $projectId;

    #[IsString]
    #[NotEmpty]
    public string $status;

    // No #[IsString] on these four: they're nullable, and Dto's attribute
    // validators run against the raw value with no null-skip for nullable
    // properties (see CreateProjectDto::$url for the same reasoning).
    public ?string $quoteId = null;

    public ?string $issueDate = null;

    public ?string $dueDate = null;

    public ?string $discountType = null;

    public float $discountValue = 0.0;

    #[IsString]
    public string $notes = '';
}
