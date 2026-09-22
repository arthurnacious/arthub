<?php

declare(strict_types=1);

use Hyperdrive\Config\Config;
use Tests\Doubles\FakeMailer;

it('sends a password reset email with the token in the URL', function () {
    Config::set('cors.allowed_origin', 'https://app.example.com');

    $mailer = new FakeMailer;
    $mailer->sendPasswordReset('user@example.com', 'Ada Lovelace', 'a-token-123');

    expect($mailer->sent)->toHaveCount(1);

    $sent = $mailer->sent[0];

    expect($sent['to_email'])->toBe('user@example.com')
        ->and($sent['to_name'])->toBe('Ada Lovelace')
        ->and($sent['subject'])->toBe('Reset your Arthub password')
        ->and($sent['body'])->toContain('https://app.example.com/reset-password?token=a-token-123');
});

it('url-encodes characters in the token that are not URL-safe', function () {
    Config::set('cors.allowed_origin', 'https://app.example.com');

    $mailer = new FakeMailer;
    $mailer->sendPasswordReset('user@example.com', 'Ada', 'has+plus&and=signs');

    expect($mailer->sent[0]['body'])->toContain(urlencode('has+plus&and=signs'))
        ->and($mailer->sent[0]['body'])->not->toContain('has+plus&and=signs');
});
