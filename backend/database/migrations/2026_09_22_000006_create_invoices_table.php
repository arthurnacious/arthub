<?php

declare(strict_types=1);

use App\Database\Migration;

// Invoices: what actually gets billed and paid. `quote_id` is nullable -
// set when an invoice was generated from an accepted quote, left null for
// an invoice created directly against a project with no quote first.
return new class ($pdo) extends Migration {
    public function up(): void
    {
        $this->pdo->exec(<<<SQL
            CREATE TYPE invoice_status AS ENUM ('draft', 'sent', 'paid', 'overdue', 'cancelled');

            CREATE TABLE invoices (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
                quote_id INTEGER NULL REFERENCES quotes (id) ON DELETE SET NULL,
                invoice_number VARCHAR(50) NOT NULL UNIQUE,
                status invoice_status NOT NULL DEFAULT 'draft',
                issue_date DATE NULL,
                due_date DATE NULL,
                discount_type discount_type NULL,
                discount_value NUMERIC(12, 2) NOT NULL DEFAULT 0,
                paid_at TIMESTAMPTZ NULL,
                notes TEXT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX invoices_project_id_index ON invoices (project_id);
            CREATE INDEX invoices_quote_id_index ON invoices (quote_id);
        SQL);
    }

    public function down(): void
    {
        $this->pdo->exec(<<<SQL
            DROP TABLE IF EXISTS invoices;
            DROP TYPE IF EXISTS invoice_status;
        SQL);
    }
};
