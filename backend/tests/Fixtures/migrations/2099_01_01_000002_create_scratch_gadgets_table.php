<?php

declare(strict_types=1);

use App\Database\Migration;

// See 2099_01_01_000001_create_scratch_widgets_table.php.
return new class ($pdo) extends Migration {
    public function up(): void
    {
        $this->pdo->exec('CREATE TABLE scratch_gadgets (id SERIAL PRIMARY KEY, name VARCHAR(255))');
    }

    public function down(): void
    {
        $this->pdo->exec('DROP TABLE IF EXISTS scratch_gadgets');
    }
};
