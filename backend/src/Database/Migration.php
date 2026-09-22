<?php

declare(strict_types=1);

namespace App\Database;

use PDO;

// Every migration file returns an instance of this. up() applies the
// change, down() reverses it. Migrations talk to Postgres directly with
// raw SQL via PDO - no schema-builder DSL - since the schema is small
// enough that plain SQL stays readable, and it means no query-builder
// dependency to learn or maintain.
abstract class Migration
{
    public function __construct(protected PDO $pdo)
    {
    }

    abstract public function up(): void;

    abstract public function down(): void;
}
