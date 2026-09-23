<?php

declare(strict_types=1);

use App\Database\Migration;

// category is free text (not an enum) so new ones don't need a migration -
// the frontend's filter list is just `SELECT DISTINCT category`. url is
// only meaningful for a published website/web-app; for something like a
// logo there's nothing to link to, so it's nullable and the frontend just
// shows the image gallery when it's absent.
//
// project_images: a project can have many images (screenshots, logo
// variations, etc.) - stored on disk under storage/app/projects/{id}/,
// served via the public/storage symlink. `path` is relative to that
// symlink (e.g. "projects/<id>/<filename>.png"), not an absolute path.
return new class ($pdo) extends Migration {
    public function up(): void
    {
        $this->pdo->exec(<<<SQL
            ALTER TABLE projects
                ADD COLUMN category VARCHAR(100) NOT NULL DEFAULT 'general',
                ADD COLUMN url VARCHAR(500) NULL;

            CREATE TABLE project_images (
                id UUID PRIMARY KEY DEFAULT uuidv7(),
                project_id UUID NOT NULL REFERENCES projects (id) ON DELETE CASCADE,
                path VARCHAR(500) NOT NULL,
                sort_order INT NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX project_images_project_id_index ON project_images (project_id);
        SQL);
    }

    public function down(): void
    {
        $this->pdo->exec(<<<SQL
            DROP TABLE IF EXISTS project_images;
            ALTER TABLE projects
                DROP COLUMN IF EXISTS category,
                DROP COLUMN IF EXISTS url;
        SQL);
    }
};
