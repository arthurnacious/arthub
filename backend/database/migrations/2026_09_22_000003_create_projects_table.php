<?php

declare(strict_types=1);

use App\Database\Migration;

// Projects: the actual work a client asked for. Quotes and invoices both
// hang off a project rather than directly off a client, since one client
// can commission several separate projects over time.
return new class ($pdo) extends Migration {
    public function up(): void
    {
        $this->pdo->exec(<<<SQL
            CREATE TYPE project_status AS ENUM ('active', 'completed', 'archived');

            CREATE TABLE projects (
                id SERIAL PRIMARY KEY,
                client_id INTEGER NOT NULL REFERENCES clients (id) ON DELETE CASCADE,
                name VARCHAR(255) NOT NULL,
                description TEXT NULL,
                status project_status NOT NULL DEFAULT 'active',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX projects_client_id_index ON projects (client_id);
        SQL);
    }

    public function down(): void
    {
        $this->pdo->exec(<<<SQL
            DROP TABLE IF EXISTS projects;
            DROP TYPE IF EXISTS project_status;
        SQL);
    }
};
