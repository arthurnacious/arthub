<?php

declare(strict_types=1);

use App\Database\Migration;

// Fixture migration used only by tests/Database/MigratorTest.php to
// exercise Migrator's up()/down() and tracking behaviour without touching
// the real application schema. The 2099 date prefix keeps it unmistakably
// separate from real migrations in database/migrations/.
return new class ($pdo) extends Migration {
    public function up(): void
    {
        $this->pdo->exec('CREATE TABLE scratch_widgets (id SERIAL PRIMARY KEY, name VARCHAR(255))');
    }

    public function down(): void
    {
        $this->pdo->exec('DROP TABLE IF EXISTS scratch_widgets');
    }
};
