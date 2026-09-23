<?php

declare(strict_types=1);

namespace App\Modules\Auth\Services;

use App\Database\Connection;
use App\Events\PasswordChanged;
use App\Events\PasswordWasReset;
use App\Events\ProfileUpdated;
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
     * Returns null if $email is already taken by a *different* user.
     *
     * @return array<string, mixed>|null the updated user, or null on a
     *                                    duplicate email
     */
    public function updateProfile(string $userId, string $name, string $email): ?array
    {
        $existing = $this->connection->pdo()->prepare(
            'SELECT 1 FROM users WHERE email = :email AND id != :id',
        );
        $existing->execute(['email' => $email, 'id' => $userId]);
        if ($existing->fetchColumn() !== false) {
            return null;
        }

        $statement = $this->connection->pdo()->prepare(
            'UPDATE users SET name = :name, email = :email, updated_at = NOW() WHERE id = :id',
        );
        $statement->execute(['name' => $name, 'email' => $email, 'id' => $userId]);

        $this->events->dispatch(new ProfileUpdated(userId: $userId, name: $name));

        return $this->findById($userId);
    }

    /**
     * Returns false if $userId doesn't exist or $currentPassword is wrong.
     */
    public function changePassword(string $userId, string $currentPassword, string $newPassword): bool
    {
        $user = $this->findById($userId);
        if ($user === null || !password_verify($currentPassword, $user['password_hash'])) {
            return false;
        }

        $statement = $this->connection->pdo()->prepare(
            'UPDATE users SET password_hash = :hash, updated_at = NOW() WHERE id = :id',
        );
        $statement->execute([
            'hash' => password_hash($newPassword, PASSWORD_BCRYPT),
            'id' => $userId,
        ]);

        $this->events->dispatch(new PasswordChanged(userId: $userId));

        return true;
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
