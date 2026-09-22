<?php

declare(strict_types=1);

namespace App\Modules\Auth\Dto;

use Hyperdrive\Http\Dto;
use Hyperdrive\Http\Dto\Validation\IsEmail;

class ForgotPasswordDto extends Dto
{
    #[IsEmail]
    public string $email;
}
