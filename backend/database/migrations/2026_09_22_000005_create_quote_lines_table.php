<?php

declare(strict_types=1);

use App\Database\Migration;

// Individual line items on a quote - what's being billed, how many, at
// what price, and each line's own optional discount (on top of the
// quote-level discount from the previous migration).
return new class ($pdo) extends Migration {
    public function up(): void
    {
        $this->pdo->exec(<<<SQL
            CREATE TABLE quote_lines (
                id UUID PRIMARY KEY DEFAULT uuidv7(),
                quote_id UUID NOT NULL REFERENCES quotes (id) ON DELETE CASCADE,
                description TEXT NOT NULL,
                quantity NUMERIC(12, 2) NOT NULL DEFAULT 1,
                unit_price NUMERIC(12, 2) NOT NULL DEFAULT 0,
                discount_type discount_type NULL,
                discount_value NUMERIC(12, 2) NOT NULL DEFAULT 0,
                position INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX quote_lines_quote_id_index ON quote_lines (quote_id);
        SQL);
    }

    public function down(): void
    {
        $this->pdo->exec('DROP TABLE IF EXISTS quote_lines');
    }
};
