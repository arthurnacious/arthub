<?php

declare(strict_types=1);

use App\Database\Migration;

// Quotes: an estimate sent to the client for a project. Once accepted, an
// invoice can be generated from it (see invoices.quote_id in a later
// migration). `discount_type`/`discount_value` here are an OVERALL
// discount on the whole quote; individual lines can also carry their own
// discount (see quote_lines) - both levels are supported on purpose.
return new class ($pdo) extends Migration {
    public function up(): void
    {
        $this->pdo->exec(<<<SQL
            CREATE TYPE discount_type AS ENUM ('percent', 'fixed');
            CREATE TYPE quote_status AS ENUM ('draft', 'sent', 'accepted', 'rejected', 'expired');

            CREATE TABLE quotes (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
                quote_number VARCHAR(50) NOT NULL UNIQUE,
                status quote_status NOT NULL DEFAULT 'draft',
                issue_date DATE NULL,
                expiry_date DATE NULL,
                discount_type discount_type NULL,
                discount_value NUMERIC(12, 2) NOT NULL DEFAULT 0,
                notes TEXT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX quotes_project_id_index ON quotes (project_id);
        SQL);
    }

    public function down(): void
    {
        $this->pdo->exec(<<<SQL
            DROP TABLE IF EXISTS quotes;
            DROP TYPE IF EXISTS quote_status;
            DROP TYPE IF EXISTS discount_type;
        SQL);
    }
};
