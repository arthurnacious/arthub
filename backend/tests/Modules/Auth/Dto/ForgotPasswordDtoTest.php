<?php

declare(strict_types=1);

use App\Modules\Auth\Dto\ForgotPasswordDto;
use Hyperdrive\Http\Dto\Validation\ValidationException;

it('accepts a valid email', function () {
    $dto = new ForgotPasswordDto(['email' => 'user@example.com']);

    expect($dto->email)->toBe('user@example.com');
});

it('rejects an invalid email', function () {
    expect(fn () => new ForgotPasswordDto(['email' => 'not-an-email']))
        ->toThrow(ValidationException::class);
});
