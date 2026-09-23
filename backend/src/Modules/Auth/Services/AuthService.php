<?php

declare(strict_types=1);

namespace App\Modules\Auth\Services;

use App\Database\Connection;
use App\Events\PasswordWasReset;
use App\Events\UserLoggedIn;
use Hyperdrive\Events\EventDispatcher;
use Hyperdrive\Security\JwtService;
use Throwable;

class AuthService
{
    public function __construct(
        private readonly Connection $connection,
        private readonly JwtService $jwt,
        private readonly EventDispatcher $events,
    ) {
    }

    /**
     * Verify credentials and, on success, return a session JWT.
     */
    public function attempt(string $email, string $password): ?string
    {
        $user = $this->findByEmail($email);

        if ($user === null || !password_verify($password, $user['password_hash'])) {
            return null;
        }

        $this->events->dispatch(new UserLoggedIn(userId: $user['id'], name: $user['name']));

        return $this->jwt->encode([
            'sub' => $user['id'], // UUID
            'email' => $user['email'],
            'role' => $user['role'],
        ]);
    }

    /**
     * @return array<string, mixed>|null
     */
    public function findByEmail(string $email): ?array
    {
        $statement = $this->connection->pdo()->prepare('SELECT * FROM users WHERE email = :email');
        $statement->execute(['email' => $email]);
        $user = $statement->fetch();

        return $user !== false ? $user : null;
    }

    /**
     * @return array<string, mixed>|null
     */
    public function findById(string $id): ?array
    {
        $statement = $this->connection->pdo()->prepare('SELECT * FROM users WHERE id = :id');
        $statement->execute(['id' => $id]);
        $user = $statement->fetch();

        return $user !== false ? $user : null;
    }

    /**
     * A short-lived, self-contained reset token - no database table needed,
     * the JWT's own signature and expiry do the work. The 'purpose' claim
     * stops it being reused as a normal session token (see
     * AuthenticateMiddleware, which rejects any token carrying one).
     *
     * @param array<string, mixed> $user
     */
    public function createPasswordResetToken(array $user): string
    {
        return $this->jwt->encode([
            'sub' => $user['id'], // UUID
            'purpose' => 'password-reset',
            'exp' => time() + 900, // 15 minutes
        ]);
    }

    public function resetPassword(string $token, string $newPassword): bool
    {
        try {
            $payload = $this->jwt->verify($token);
        } catch (Throwable) {
            return false;
        }

        if (($payload['purpose'] ?? null) !== 'password-reset') {
            return false;
        }

        $statement = $this->connection->pdo()->prepare(
            'UPDATE users SET password_hash = :hash, updated_at = NOW() WHERE id = :id'
        );
        $statement->execute([
            'hash' => password_hash($newPassword, PASSWORD_BCRYPT),
            'id' => $payload['sub'],
        ]);

        $this->events->dispatch(new PasswordWasReset(userId: $payload['sub']));

        return true;
    }
}
