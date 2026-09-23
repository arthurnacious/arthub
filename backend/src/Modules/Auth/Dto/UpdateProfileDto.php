<?php

declare(strict_types=1);

namespace App\Modules\Auth\Dto;

use Hyperdrive\Http\Dto;
use Hyperdrive\Http\Dto\Validation\IsEmail;
use Hyperdrive\Http\Dto\Validation\IsString;
use Hyperdrive\Http\Dto\Validation\NotEmpty;

class UpdateProfileDto extends Dto
{
    #[IsString]
    #[NotEmpty]
    public string $name;

    #[IsEmail]
    public string $email;
}
