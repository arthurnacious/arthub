<?php

declare(strict_types=1);

namespace App\Modules\Activity\Listeners;

use App\Events\ClientCreated;
use App\Events\ClientDeleted;
use App\Events\ClientUpdated;
use App\Events\PasswordWasReset;
use App\Events\ProjectCreated;
use App\Events\ProjectDeleted;
use App\Events\ProjectUpdated;
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
}
