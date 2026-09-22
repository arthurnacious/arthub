<?php

declare(strict_types=1);

use App\Modules\Auth\Dto\LoginDto;
use Hyperdrive\Http\Dto\Validation\ValidationException;

it('accepts a valid email and non-empty password', function () {
    $dto = new LoginDto(['email' => 'user@example.com', 'password' => 'secret']);

    expect($dto->email)->toBe('user@example.com')
        ->and($dto->password)->toBe('secret');
});

it('rejects an invalid email', function () {
    expect(fn () => new LoginDto(['email' => 'not-an-email', 'password' => 'secret']))
        ->toThrow(ValidationException::class);
});

it('rejects an empty password', function () {
    try {
        new LoginDto(['email' => 'user@example.com', 'password' => '']);
        expect(false)->toBeTrue('expected a ValidationException');
    } catch (ValidationException $e) {
        expect($e->getErrors())->toHaveKey('password');
    }
});
