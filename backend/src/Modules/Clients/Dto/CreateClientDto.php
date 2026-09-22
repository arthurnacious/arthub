<?php

declare(strict_types=1);

namespace App\Modules\Clients\Dto;

use Hyperdrive\Http\Dto;
use Hyperdrive\Http\Dto\Validation\IsEmail;
use Hyperdrive\Http\Dto\Validation\IsString;
use Hyperdrive\Http\Dto\Validation\NotEmpty;

// Status isn't validated here as an enum - #[ValidateWith] only actually
// runs through DtoFactory (the real request pipeline), not when a DTO is
// constructed directly, which is how this framework's own tests exercise
// controllers (see e.g. AuthControllerTest). ClientsController checks it
// manually instead, so the check is covered by a real, direct-call test.
class CreateClientDto extends Dto
{
    #[IsString]
    #[NotEmpty]
    public string $company;

    #[IsString]
    #[NotEmpty]
    public string $contact;

    #[IsEmail]
    public string $email;

    #[IsString]
    #[NotEmpty]
    public string $status;
}
