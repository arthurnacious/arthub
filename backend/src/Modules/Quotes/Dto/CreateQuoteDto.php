<?php

declare(strict_types=1);

namespace App\Modules\Quotes\Dto;

use Hyperdrive\Http\Dto;
use Hyperdrive\Http\Dto\Validation\IsString;
use Hyperdrive\Http\Dto\Validation\NotEmpty;

// status/discountType aren't validated here as enums, same reasoning as
// Modules\Projects\Dto\CreateProjectDto: #[ValidateWith] only runs through
// DtoFactory (the real request pipeline), not a direct `new` - which is
// how this framework's own tests exercise controllers. QuotesController
// checks them manually instead.
//
// Line items aren't part of this Dto at all - they're a nested array
// ($request->json('lineItems')), which the Dto system doesn't validate,
// so QuotesController validates them by hand too.
class CreateQuoteDto extends Dto
{
    #[IsString]
    #[NotEmpty]
    public string $projectId;

    #[IsString]
    #[NotEmpty]
    public string $status;

    // No #[IsString] on these three: they're nullable, and Dto's
    // attribute validators run against the raw value with no null-skip
    // for nullable properties (see CreateProjectDto::$url for the same
    // reasoning).
    public ?string $issueDate = null;

    public ?string $expiryDate = null;

    public ?string $discountType = null;

    public float $discountValue = 0.0;

    #[IsString]
    public string $notes = '';
}
