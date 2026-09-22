<?php

declare(strict_types=1);

// Read by Hyperdrive's JwtService (Hyperdrive\Security\JwtService).
return [
    'jwt' => [
        'secret' => $_ENV['JWT_SECRET'] ?? 'change-me-to-a-long-random-string',
        'algorithm' => 'HS256',
        'expiry' => 60 * 60 * 12, // 12 hours
    ],
];
