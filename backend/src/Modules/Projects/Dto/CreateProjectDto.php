<?php

declare(strict_types=1);

namespace App\Modules\Projects\Dto;

use Hyperdrive\Http\Dto;
use Hyperdrive\Http\Dto\Validation\IsString;
use Hyperdrive\Http\Dto\Validation\NotEmpty;

// status isn't validated here as an enum, same reasoning as
// Modules\Clients\Dto\CreateClientDto: #[ValidateWith] only runs through
// DtoFactory (the real request pipeline), not a direct `new` - which is
// how this framework's own tests exercise controllers. ProjectsController
// checks it manually instead.
class CreateProjectDto extends Dto
{
    #[IsString]
    #[NotEmpty]
    public string $clientId;

    #[IsString]
    #[NotEmpty]
    public string $name;

    #[IsString]
    public string $description = '';

    #[IsString]
    #[NotEmpty]
    public string $category;

    // No #[IsString] here: Dto's attribute validators run against the raw
    // value with no null-skip for nullable properties, so IsString would
    // reject this field's own default/absent value.
    public ?string $url = null;

    #[IsString]
    #[NotEmpty]
    public string $status;
}
