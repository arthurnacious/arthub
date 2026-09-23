<?php

declare(strict_types=1);

use App\Modules\Auth\AuthController;
use App\Modules\Auth\Dto\ChangePasswordDto;
use App\Modules\Auth\Dto\ForgotPasswordDto;
use App\Modules\Auth\Dto\LoginDto;
use App\Modules\Auth\Dto\ResetPasswordDto;
use App\Modules\Auth\Dto\UpdateProfileDto;
use App\Modules\Auth\Services\AuthService;
use Hyperdrive\Http\Request;
use Hyperdrive\Security\JwtService;
use Tests\Doubles\FakeMailer;

function make_auth_controller(): array
{
    $mailer = new FakeMailer;
    $auth = new AuthService(test_connection(), new JwtService, make_event_dispatcher());

    return [new AuthController($auth, $mailer), $auth, $mailer];
}

it('logs in with correct credentials and returns a token', function () {
    create_test_user(email: 'login@example.com', password: 'correct-password');
    [$controller] = make_auth_controller();

    $response = $controller->login(new LoginDto(['email' => 'login@example.com', 'password' => 'correct-password']));
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body)->toHaveKey('token');
});

it('rejects login with the wrong password', function () {
    create_test_user(email: 'login@example.com', password: 'correct-password');
    [$controller] = make_auth_controller();

    $response = $controller->login(new LoginDto(['email' => 'login@example.com', 'password' => 'wrong']));
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(401)
        ->and($body['error'])->toBe('Invalid credentials');
});

it('sends a reset email for a known address and does not leak whether the email exists', function () {
    $userId = create_test_user(email: 'forgot@example.com');
    [$controller, , $mailer] = make_auth_controller();

    $known = $controller->forgotPassword(new ForgotPasswordDto(['email' => 'forgot@example.com']));
    $unknown = $controller->forgotPassword(new ForgotPasswordDto(['email' => 'nobody@example.com']));

    expect($known->getStatusCode())->toBe(200)
        ->and($unknown->getStatusCode())->toBe(200)
        ->and(json_decode($known->getContent(), true)['message'])
        ->toBe(json_decode($unknown->getContent(), true)['message'])
        ->and($mailer->sent)->toHaveCount(1)
        ->and($mailer->sent[0]['to_email'])->toBe('forgot@example.com');
});

it('resets the password with a valid token', function () {
    $userId = create_test_user(email: 'reset@example.com', password: 'old-password');
    [$controller, $auth] = make_auth_controller();
    $token = $auth->createPasswordResetToken($auth->findById($userId));

    $response = $controller->resetPassword(new ResetPasswordDto(['token' => $token, 'password' => 'new-password']));

    expect($response->getStatusCode())->toBe(200)
        ->and($auth->attempt('reset@example.com', 'new-password'))->toBeString();
});

it('rejects an invalid reset token', function () {
    [$controller] = make_auth_controller();

    $response = $controller->resetPassword(new ResetPasswordDto(['token' => 'garbage', 'password' => 'new-password']));
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(422)
        ->and($body['error'])->toBe('Invalid or expired reset token');
});

it('returns the current user for /me', function () {
    $userId = create_test_user(email: 'me@example.com', name: 'Me');
    [$controller] = make_auth_controller();

    $response = $controller->me(new Request(attributes: ['auth_user' => ['sub' => $userId]]));
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['name'])->toBe('Me')
        ->and($body['email'])->toBe('me@example.com')
        ->and($body)->not->toHaveKey('password_hash');
});

it('updates the current user\'s name and email', function () {
    $userId = create_test_user(email: 'old@example.com', name: 'Old Name');
    [$controller] = make_auth_controller();

    $response = $controller->updateMe(
        new UpdateProfileDto(['name' => 'New Name', 'email' => 'new@example.com']),
        new Request(attributes: ['auth_user' => ['sub' => $userId]]),
    );
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['name'])->toBe('New Name')
        ->and($body['email'])->toBe('new@example.com');
});

it('rejects updating to an email already used by someone else', function () {
    create_test_user(email: 'taken@example.com');
    $userId = create_test_user(email: 'mine@example.com');
    [$controller] = make_auth_controller();

    $response = $controller->updateMe(
        new UpdateProfileDto(['name' => 'X', 'email' => 'taken@example.com']),
        new Request(attributes: ['auth_user' => ['sub' => $userId]]),
    );

    expect($response->getStatusCode())->toBe(422);
});

it('changes the current user\'s password given the correct current one', function () {
    $userId = create_test_user(email: 'pw@example.com', password: 'old-password');
    [$controller, $auth] = make_auth_controller();

    $response = $controller->changePassword(
        new ChangePasswordDto(['currentPassword' => 'old-password', 'newPassword' => 'new-password']),
        new Request(attributes: ['auth_user' => ['sub' => $userId]]),
    );

    expect($response->getStatusCode())->toBe(200)
        ->and($auth->attempt('pw@example.com', 'new-password'))->toBeString();
});

it('rejects changing the password with the wrong current password', function () {
    $userId = create_test_user(email: 'pw@example.com', password: 'old-password');
    [$controller] = make_auth_controller();

    $response = $controller->changePassword(
        new ChangePasswordDto(['currentPassword' => 'wrong', 'newPassword' => 'new-password']),
        new Request(attributes: ['auth_user' => ['sub' => $userId]]),
    );
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(422)
        ->and($body['error'])->toBe('Current password is incorrect');
});
