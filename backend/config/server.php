<?php

declare(strict_types=1);

// Serves public/ straight from OpenSwoole (uploaded project images live
// under storage/app, reachable at /storage/... via the symlink the
// storage:link command creates in public/).
return [
    'static' => [
        'enabled' => true,
        'document_root' => __DIR__ . '/../public',
    ],
];
