<?php

declare(strict_types=1);

namespace App\Modules\Activity\Listeners;

use App\Events\ClientCreated;
use App\Events\ClientDeleted;
use App\Events\ClientUpdated;
use App\Events\InvoiceCreated;
use App\Events\InvoiceDeleted;
use App\Events\InvoicePaid;
use App\Events\InvoiceUpdated;
use App\Events\PasswordChanged;
use App\Events\PasswordWasReset;
use App\Events\ProfileUpdated;
use App\Events\ProjectCreated;
use App\Events\ProjectDeleted;
use App\Events\ProjectUpdated;
use App\Events\QuoteCreated;
use App\Events\QuoteDeleted;
use App\Events\QuoteUpdated;
use App\Events\UserLoggedIn;
use App\Support\ActivityLogger;
use Hyperdrive\Attributes\Events\Listener;

// Translates domain events into activity_log rows. This is the only class
// that knows both "what happened" (the event) and "how to describe it in
// the log" - everywhere else just dispatches an event and moves on,
// without knowing activity logging exists.
class ActivityLogListener
{
    public function __construct(private readonly ActivityLogger $logger)
    {
    }

    #[Listener(ClientCreated::class)]
    public function onClientCreated(ClientCreated $event): void
    {
        $this->logger->log(
            action: 'client.created',
            description: "Created client {$event->company}",
            userId: $event->userId,
            subjectType: 'client',
            subjectId: $event->clientId,
        );
    }

    #[Listener(ClientUpdated::class)]
    public function onClientUpdated(ClientUpdated $event): void
    {
        $this->logger->log(
            action: 'client.updated',
            description: "Updated client {$event->company}",
            userId: $event->userId,
            subjectType: 'client',
            subjectId: $event->clientId,
        );
    }

    #[Listener(ClientDeleted::class)]
    public function onClientDeleted(ClientDeleted $event): void
    {
        $this->logger->log(
            action: 'client.deleted',
            description: "Deleted client {$event->company}",
            userId: $event->userId,
            subjectType: 'client',
            subjectId: $event->clientId,
        );
    }

    #[Listener(ProjectCreated::class)]
    public function onProjectCreated(ProjectCreated $event): void
    {
        $this->logger->log(
            action: 'project.created',
            description: "Created project {$event->name}",
            userId: $event->userId,
            subjectType: 'project',
            subjectId: $event->projectId,
        );
    }

    #[Listener(ProjectUpdated::class)]
    public function onProjectUpdated(ProjectUpdated $event): void
    {
        $this->logger->log(
            action: 'project.updated',
            description: "Updated project {$event->name}",
            userId: $event->userId,
            subjectType: 'project',
            subjectId: $event->projectId,
        );
    }

    #[Listener(ProjectDeleted::class)]
    public function onProjectDeleted(ProjectDeleted $event): void
    {
        $this->logger->log(
            action: 'project.deleted',
            description: "Deleted project {$event->name}",
            userId: $event->userId,
            subjectType: 'project',
            subjectId: $event->projectId,
        );
    }

    #[Listener(QuoteCreated::class)]
    public function onQuoteCreated(QuoteCreated $event): void
    {
        $this->logger->log(
            action: 'quote.created',
            description: "Created quote {$event->quoteNumber}",
            userId: $event->userId,
            subjectType: 'quote',
            subjectId: $event->quoteId,
        );
    }

    #[Listener(QuoteUpdated::class)]
    public function onQuoteUpdated(QuoteUpdated $event): void
    {
        $this->logger->log(
            action: 'quote.updated',
            description: "Updated quote {$event->quoteNumber}",
            userId: $event->userId,
            subjectType: 'quote',
            subjectId: $event->quoteId,
        );
    }

    #[Listener(QuoteDeleted::class)]
    public function onQuoteDeleted(QuoteDeleted $event): void
    {
        $this->logger->log(
            action: 'quote.deleted',
            description: "Deleted quote {$event->quoteNumber}",
            userId: $event->userId,
            subjectType: 'quote',
            subjectId: $event->quoteId,
        );
    }

    #[Listener(InvoiceCreated::class)]
    public function onInvoiceCreated(InvoiceCreated $event): void
    {
        $this->logger->log(
            action: 'invoice.created',
            description: "Created invoice {$event->invoiceNumber}",
            userId: $event->userId,
            subjectType: 'invoice',
            subjectId: $event->invoiceId,
        );
    }

    #[Listener(InvoiceUpdated::class)]
    public function onInvoiceUpdated(InvoiceUpdated $event): void
    {
        $this->logger->log(
            action: 'invoice.updated',
            description: "Updated invoice {$event->invoiceNumber}",
            userId: $event->userId,
            subjectType: 'invoice',
            subjectId: $event->invoiceId,
        );
    }

    #[Listener(InvoiceDeleted::class)]
    public function onInvoiceDeleted(InvoiceDeleted $event): void
    {
        $this->logger->log(
            action: 'invoice.deleted',
            description: "Deleted invoice {$event->invoiceNumber}",
            userId: $event->userId,
            subjectType: 'invoice',
            subjectId: $event->invoiceId,
        );
    }

    #[Listener(InvoicePaid::class)]
    public function onInvoicePaid(InvoicePaid $event): void
    {
        $this->logger->log(
            action: 'invoice.paid',
            description: "Invoice {$event->invoiceNumber} marked paid",
            userId: $event->userId,
            subjectType: 'invoice',
            subjectId: $event->invoiceId,
            metadata: ['total' => $event->total],
        );
    }

    #[Listener(UserLoggedIn::class)]
    public function onUserLoggedIn(UserLoggedIn $event): void
    {
        $this->logger->log('auth.login', "{$event->name} logged in", $event->userId);
    }

    #[Listener(PasswordWasReset::class)]
    public function onPasswordWasReset(PasswordWasReset $event): void
    {
        $this->logger->log('auth.password_reset', 'Password was reset', $event->userId);
    }

    #[Listener(ProfileUpdated::class)]
    public function onProfileUpdated(ProfileUpdated $event): void
    {
        $this->logger->log('auth.profile_updated', "Updated profile for {$event->name}", $event->userId);
    }

    #[Listener(PasswordChanged::class)]
    public function onPasswordChanged(PasswordChanged $event): void
    {
        $this->logger->log('auth.password_changed', 'Password was changed', $event->userId);
    }
}
