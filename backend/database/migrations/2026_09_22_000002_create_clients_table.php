<?php

declare(strict_types=1);

use App\Database\Migration;

// Clients: the companies that hire you. A client can later be linked to a
// login (users.client_id) for a client portal, but that's not required -
// most clients will just be a record you bill against with no login at
// all.
//
// id is a UUIDv7 (see 2026_09_22_000001_create_users_table.php for why) -
// every table's id in this schema is, so all of them stay consistent and
// none of the FKs between them end up mismatched.
return new class ($pdo) extends Migration {
    public function up(): void
    {
        $this->pdo->exec(<<<SQL
            CREATE TABLE clients (
                id UUID PRIMARY KEY DEFAULT uuidv7(),
                company_name VARCHAR(255) NOT NULL,
                contact_name VARCHAR(255) NULL,
                email VARCHAR(255) NULL,
                phone VARCHAR(50) NULL,
                address TEXT NULL,
                notes TEXT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            ALTER TABLE users
                ADD CONSTRAINT users_client_id_fkey
                FOREIGN KEY (client_id) REFERENCES clients (id)
                ON DELETE SET NULL;
        SQL);
    }

    public function down(): void
    {
        $this->pdo->exec(<<<SQL
            ALTER TABLE users DROP CONSTRAINT IF EXISTS users_client_id_fkey;
            DROP TABLE IF EXISTS clients;
        SQL);
    }
};
