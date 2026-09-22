<?php

declare(strict_types=1);

namespace App\Mail;

use Hyperdrive\Config\Config;
use PHPMailer\PHPMailer\Exception as PHPMailerException;
use PHPMailer\PHPMailer\PHPMailer;
use RuntimeException;

// Thin wrapper around PHPMailer so the rest of the app calls small,
// purpose-named methods (e.g. sendPasswordReset) instead of touching
// PHPMailer directly everywhere - more mail types get their own method
// here as the app grows (e.g. sendInvoice once invoices exist).
class Mailer
{
    public function sendPasswordReset(string $toEmail, string $toName, string $token): void
    {
        $resetUrl = rtrim((string) Config::get('cors.allowed_origin', ''), '/')
            . '/reset-password?token=' . urlencode($token);

        $this->send(
            $toEmail,
            $toName,
            'Reset your Arthub password',
            "Hi {$toName},\n\n"
                . "Use the link below to reset your password. It expires in 15 minutes.\n\n"
                . "{$resetUrl}\n\n"
                . "If you didn't request this, you can ignore this email.",
        );
    }

    private function send(string $toEmail, string $toName, string $subject, string $body): void
    {
        $config = Config::get('mail');

        $mailer = new PHPMailer(true);

        try {
            $mailer->isSMTP();
            $mailer->Host = $config['host'];
            $mailer->Port = $config['port'];
            $mailer->SMTPAuth = $config['username'] !== '';
            $mailer->Username = $config['username'];
            $mailer->Password = $config['password'];
            $mailer->SMTPSecure = $config['encryption'];

            $mailer->setFrom($config['from_address'], $config['from_name']);
            $mailer->addAddress($toEmail, $toName);

            $mailer->Subject = $subject;
            $mailer->Body = $body;
            $mailer->isHTML(false);

            $mailer->send();
        } catch (PHPMailerException $e) {
            throw new RuntimeException("Failed to send email: {$mailer->ErrorInfo}", previous: $e);
        }
    }
}
