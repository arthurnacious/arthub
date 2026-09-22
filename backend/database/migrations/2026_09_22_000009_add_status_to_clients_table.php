<?php

declare(strict_types=1);

use App\Database\Migration;

// The frontend's client list groups/badges by status (active/prospect/
// inactive) - that needs a real column, it wasn't part of the original
// clients table.
return new class ($pdo) extends Migration {
    public function up(): void
    {
        $this->pdo->exec(<<<SQL
            CREATE TYPE client_status AS ENUM ('active', 'prospect', 'inactive');

            ALTER TABLE clients
                ADD COLUMN status client_status NOT NULL DEFAULT 'prospect';
        SQL);
    }

    public function down(): void
    {
        $this->pdo->exec(<<<SQL
            ALTER TABLE clients DROP COLUMN IF EXISTS status;
            DROP TYPE IF EXISTS client_status;
        SQL);
    }
};
