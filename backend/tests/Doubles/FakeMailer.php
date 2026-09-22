<?php

declare(strict_types=1);

namespace Tests\Doubles;

use App\Mail\Mailer;

// A Mailer that records what it was asked to send instead of touching
// real SMTP - used by tests that exercise code paths which send email
// (e.g. AuthController::forgotPassword).
class FakeMailer extends Mailer
{
    /** @var array<int, array{to_email: string, to_name: string, subject: string, body: string}> */
    public array $sent = [];

    protected function send(string $toEmail, string $toName, string $subject, string $body): void
    {
        $this->sent[] = [
            'to_email' => $toEmail,
            'to_name' => $toName,
            'subject' => $subject,
            'body' => $body,
        ];
    }
}
