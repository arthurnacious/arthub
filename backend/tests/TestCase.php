<?php

declare(strict_types=1);

namespace Tests;

use Hyperdrive\Config\Config;
use PHPUnit\Framework\TestCase as BaseTestCase;

// Bound to every test via `uses(TestCase::class)->in(__DIR__)` in Pest.php.
//
// A bare top-level beforeEach()/afterEach() in Pest.php looks like it
// should apply globally, but it doesn't: Pest registers those hooks keyed
// by Pest.php's own filename, while each test looks its hooks up by its
// own filename - the two never match unless the hook was registered
// through uses()->in(), which does the per-file fan-out properly. Hence
// this class instead of bare beforeEach()/afterEach() calls.
abstract class TestCase extends BaseTestCase
{
    protected function setUp(): void
    {
        parent::setUp();

        // Config is a process-wide singleton, so any Config::set() a test
        // makes (e.g. to control cors.allowed_origin) would otherwise leak
        // into every test that runs after it.
        Config::clear();

        test_connection()->pdo()->beginTransaction();
    }

    protected function tearDown(): void
    {
        $pdo = test_connection()->pdo();

        if ($pdo->inTransaction()) {
            $pdo->rollBack();
        }

        parent::tearDown();
    }
}
