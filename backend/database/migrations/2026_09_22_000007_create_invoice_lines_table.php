<?php

declare(strict_types=1);

use App\Database\Migration;

// Individual line items on an invoice - same shape as quote_lines, kept
// as a separate table (rather than one shared "lines" table) so an
// invoice generated from a quote gets its own independent copy of the
// lines, which can then be edited without altering the original quote.
return new class ($pdo) extends Migration {
    public function up(): void
    {
        $this->pdo->exec(<<<SQL
            CREATE TABLE invoice_lines (
                id SERIAL PRIMARY KEY,
                invoice_id INTEGER NOT NULL REFERENCES invoices (id) ON DELETE CASCADE,
                description TEXT NOT NULL,
                quantity NUMERIC(12, 2) NOT NULL DEFAULT 1,
                unit_price NUMERIC(12, 2) NOT NULL DEFAULT 0,
                discount_type discount_type NULL,
                discount_value NUMERIC(12, 2) NOT NULL DEFAULT 0,
                position INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX invoice_lines_invoice_id_index ON invoice_lines (invoice_id);
        SQL);
    }

    public function down(): void
    {
        $this->pdo->exec('DROP TABLE IF EXISTS invoice_lines');
    }
};
