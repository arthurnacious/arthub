<?php

declare(strict_types=1);

// SMTP settings PHPMailer::App\Mail\Mailer reads. See .env.example.
return [
    'host' => $_ENV['MAIL_HOST'] ?? 'localhost',
    'port' => (int) ($_ENV['MAIL_PORT'] ?? 2525),
    'username' => $_ENV['MAIL_USERNAME'] ?? '',
    'password' => $_ENV['MAIL_PASSWORD'] ?? '',
    'encryption' => $_ENV['MAIL_ENCRYPTION'] ?? 'tls',
    'from_address' => $_ENV['MAIL_FROM_ADDRESS'] ?? 'no-reply@arthub.test',
    'from_name' => $_ENV['MAIL_FROM_NAME'] ?? 'Arthub',
];
