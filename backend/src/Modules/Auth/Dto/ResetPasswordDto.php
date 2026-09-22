<?php

declare(strict_types=1);

namespace App\Modules\Auth\Dto;

use Hyperdrive\Http\Dto;
use Hyperdrive\Http\Dto\Validation\IsString;
use Hyperdrive\Http\Dto\Validation\MinLength;
use Hyperdrive\Http\Dto\Validation\NotEmpty;

class ResetPasswordDto extends Dto
{
    #[IsString]
    #[NotEmpty]
    public string $token;

    #[IsString]
    #[MinLength(8)]
    public string $password;
}
