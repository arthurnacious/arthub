<?php

declare(strict_types=1);

use App\Modules\Auth\Services\AuthService;
use Hyperdrive\Security\JwtService;

function make_auth_service(): AuthService
{
    return new AuthService(test_connection(), new JwtService, make_event_dispatcher());
}

it('returns a JWT for correct credentials', function () {
    create_test_user(email: 'auth@example.com', password: 'correct-password');

    $token = make_auth_service()->attempt('auth@example.com', 'correct-password');

    expect($token)->toBeString();

    $payload = (new JwtService)->verify($token);
    expect($payload['email'])->toBe('auth@example.com')
        ->and($payload['role'])->toBe('admin');
});

it('returns null for a wrong password', function () {
    create_test_user(email: 'auth@example.com', password: 'correct-password');

    $token = make_auth_service()->attempt('auth@example.com', 'wrong-password');

    expect($token)->toBeNull();
});

it('returns null for an unknown email', function () {
    $token = make_auth_service()->attempt('nobody@example.com', 'whatever');

    expect($token)->toBeNull();
});

it('logs an activity entry on successful login', function () {
    $userId = create_test_user(email: 'auth@example.com', password: 'correct-password');

    make_auth_service()->attempt('auth@example.com', 'correct-password');

    $row = test_connection()->pdo()
        ->query('SELECT * FROM activity_log ORDER BY id DESC LIMIT 1')
        ->fetch();

    expect($row['action'])->toBe('auth.login')
        ->and($row['user_id'])->toBe($userId);
});

it('finds a user by email', function () {
    $userId = create_test_user(email: 'find-me@example.com');

    $user = make_auth_service()->findByEmail('find-me@example.com');

    expect($user)->not->toBeNull()
        ->and($user['id'])->toBe($userId);
});

it('returns null from findByEmail for an unknown address', function () {
    expect(make_auth_service()->findByEmail('nobody@example.com'))->toBeNull();
});

it('finds a user by id', function () {
    $userId = create_test_user(email: 'find-by-id@example.com');

    $user = make_auth_service()->findById($userId);

    expect($user)->not->toBeNull()
        ->and($user['email'])->toBe('find-by-id@example.com');
});

it('creates a password reset token and resets the password with it', function () {
    $userId = create_test_user(email: 'reset@example.com', password: 'old-password');
    $service = make_auth_service();
    $user = $service->findById($userId);

    $token = $service->createPasswordResetToken($user);
    $ok = $service->resetPassword($token, 'new-password');

    expect($ok)->toBeTrue()
        ->and($service->attempt('reset@example.com', 'new-password'))->toBeString()
        ->and($service->attempt('reset@example.com', 'old-password'))->toBeNull();
});

it('logs an activity entry when a password is reset', function () {
    $userId = create_test_user(email: 'reset@example.com', password: 'old-password');
    $service = make_auth_service();
    $token = $service->createPasswordResetToken($service->findById($userId));

    $service->resetPassword($token, 'new-password');

    $row = test_connection()->pdo()
        ->query('SELECT * FROM activity_log ORDER BY id DESC LIMIT 1')
        ->fetch();

    expect($row['action'])->toBe('auth.password_reset')
        ->and($row['user_id'])->toBe($userId);
});

it('rejects a reset token that is not actually a password-reset token', function () {
    create_test_user(email: 'reset@example.com', password: 'old-password');
    $service = make_auth_service();

    // A normal session token, not one from createPasswordResetToken().
    $sessionToken = (new JwtService)->encode(['sub' => 1]);

    expect($service->resetPassword($sessionToken, 'new-password'))->toBeFalse();
});

it('rejects a garbage reset token', function () {
    expect(make_auth_service()->resetPassword('not-a-real-token', 'new-password'))->toBeFalse();
});
