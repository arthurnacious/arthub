<?php

declare(strict_types=1);

namespace App\Modules\Auth\Dto;

use Hyperdrive\Http\Dto;
use Hyperdrive\Http\Dto\Validation\IsString;
use Hyperdrive\Http\Dto\Validation\MinLength;
use Hyperdrive\Http\Dto\Validation\NotEmpty;

class ChangePasswordDto extends Dto
{
    #[IsString]
    #[NotEmpty]
    public string $currentPassword;

    #[IsString]
    #[MinLength(8)]
    public string $newPassword;
}
