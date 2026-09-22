<?php

declare(strict_types=1);

use App\Modules\Auth\Dto\ResetPasswordDto;
use Hyperdrive\Http\Dto\Validation\ValidationException;

it('accepts a token and an 8+ character password', function () {
    $dto = new ResetPasswordDto(['token' => 'abc.def.ghi', 'password' => 'longenough']);

    expect($dto->token)->toBe('abc.def.ghi')
        ->and($dto->password)->toBe('longenough');
});

it('rejects an empty token', function () {
    expect(fn () => new ResetPasswordDto(['token' => '', 'password' => 'longenough']))
        ->toThrow(ValidationException::class);
});

it('rejects a password shorter than 8 characters', function () {
    try {
        new ResetPasswordDto(['token' => 'abc.def.ghi', 'password' => 'short']);
        expect(false)->toBeTrue('expected a ValidationException');
    } catch (ValidationException $e) {
        expect($e->getErrors())->toHaveKey('password');
    }
});
