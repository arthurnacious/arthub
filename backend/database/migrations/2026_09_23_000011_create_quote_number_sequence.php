<?php

declare(strict_types=1);

use App\Database\Migration;

// Sequential, human-readable quote numbers (Q-0001, Q-0002, ...) -
// generated via QuotesController::nextQuoteNumber() using this sequence
// rather than SELECT MAX(...)+1, which would race under concurrent
// requests (OpenSwoole serves many coroutines at once). Postgres
// sequences are atomic across concurrent transactions.
return new class ($pdo) extends Migration {
    public function up(): void
    {
        $this->pdo->exec('CREATE SEQUENCE quote_number_seq START WITH 1');
    }

    public function down(): void
    {
        $this->pdo->exec('DROP SEQUENCE IF EXISTS quote_number_seq');
    }
};
