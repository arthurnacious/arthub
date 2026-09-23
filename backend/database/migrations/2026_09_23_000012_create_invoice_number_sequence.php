<?php

declare(strict_types=1);

use App\Database\Migration;

// Sequential, human-readable invoice numbers (INV-0001, INV-0002, ...) -
// same reasoning as quote_number_seq: a Postgres sequence rather than
// SELECT MAX(...)+1, which would race under OpenSwoole's concurrent
// request handling.
return new class ($pdo) extends Migration {
    public function up(): void
    {
        $this->pdo->exec('CREATE SEQUENCE invoice_number_seq START WITH 1');
    }

    public function down(): void
    {
        $this->pdo->exec('DROP SEQUENCE IF EXISTS invoice_number_seq');
    }
};
