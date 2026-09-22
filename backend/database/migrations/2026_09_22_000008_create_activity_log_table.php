<?php

declare(strict_types=1);

use App\Database\Migration;

// Feeds the terminal-style activity log in the frontend. Every
// interesting thing the backend does (client created, invoice marked
// paid, etc.) writes one row here. `metadata` holds any extra structured
// detail (e.g. old/new values) that doesn't need its own column.
return new class ($pdo) extends Migration {
    public function up(): void
    {
        $this->pdo->exec(<<<SQL
            CREATE TABLE activity_log (
                id SERIAL PRIMARY KEY,
                user_id INTEGER NULL REFERENCES users (id) ON DELETE SET NULL,
                action VARCHAR(100) NOT NULL,
                description TEXT NOT NULL,
                subject_type VARCHAR(100) NULL,
                subject_id INTEGER NULL,
                metadata JSONB NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX activity_log_created_at_index ON activity_log (created_at DESC);
        SQL);
    }

    public function down(): void
    {
        $this->pdo->exec('DROP TABLE IF EXISTS activity_log');
    }
};
