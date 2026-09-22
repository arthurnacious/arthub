<?php

declare(strict_types=1);

namespace App\Modules\Auth;

use App\Mail\Mailer;
use App\Modules\Auth\Dto\ForgotPasswordDto;
use App\Modules\Auth\Dto\LoginDto;
use App\Modules\Auth\Dto\ResetPasswordDto;
use App\Modules\Auth\Services\AuthService;
use Hyperdrive\Attributes\Http\Route;
use Hyperdrive\Attributes\Http\Verbs\Post;
use Hyperdrive\Http\Response;

#[Route('/auth')]
class AuthController
{
    public function __construct(
        private readonly AuthService $auth,
        private readonly Mailer $mailer,
    ) {
    }

    #[Post('/login')]
    public function login(LoginDto $dto): Response
    {
        $token = $this->auth->attempt($dto->email, $dto->password);

        if ($token === null) {
            return Response::json(['error' => 'Invalid credentials'], 401);
        }

        return Response::json(['token' => $token]);
    }

    #[Post('/forgot-password')]
    public function forgotPassword(ForgotPasswordDto $dto): Response
    {
        $user = $this->auth->findByEmail($dto->email);

        // Respond the same way whether or not the email is registered, so
        // this endpoint can't be used to find out who has an account.
        if ($user !== null) {
            $token = $this->auth->createPasswordResetToken($user);
            $this->mailer->sendPasswordReset($user['email'], $user['name'], $token);
        }

        return Response::json(['message' => 'If that email exists, a reset link has been sent.']);
    }

    #[Post('/reset-password')]
    public function resetPassword(ResetPasswordDto $dto): Response
    {
        if (!$this->auth->resetPassword($dto->token, $dto->password)) {
            return Response::json(['error' => 'Invalid or expired reset token'], 422);
        }

        return Response::json(['message' => 'Password updated.']);
    }
}
