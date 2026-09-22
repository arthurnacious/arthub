<?php

declare(strict_types=1);

use App\Database\Migration;

// Users: the people who can log in. `role` distinguishes the admin (you,
// for now) from a future client-portal login. `client_id` is set only for
// client-role users, linking them to the company they belong to - it has
// no foreign key yet because the clients table doesn't exist until the
// next migration; that constraint gets added there.
return new class ($pdo) extends Migration {
    public function up(): void
    {
        $this->pdo->exec(<<<SQL
            CREATE TYPE user_role AS ENUM ('admin', 'client');

            CREATE TABLE users (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL UNIQUE,
                password_hash VARCHAR(255) NOT NULL,
                role user_role NOT NULL DEFAULT 'client',
                client_id INTEGER NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        SQL);
    }

    public function down(): void
    {
        $this->pdo->exec(<<<SQL
            DROP TABLE IF EXISTS users;
            DROP TYPE IF EXISTS user_role;
        SQL);
    }
};
