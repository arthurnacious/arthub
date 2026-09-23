<?php

declare(strict_types=1);

use App\Events\ClientCreated;
use App\Events\ClientDeleted;
use App\Events\ClientUpdated;
use App\Events\PasswordWasReset;
use App\Events\ProjectCreated;
use App\Events\ProjectDeleted;
use App\Events\ProjectUpdated;
use App\Events\UserLoggedIn;
use App\Modules\Activity\Listeners\ActivityLogListener;
use App\Support\ActivityLogger;

function make_activity_log_listener(): ActivityLogListener
{
    return new ActivityLogListener(new ActivityLogger(test_connection()));
}

function latest_activity_log_row(string $action): array|false
{
    $statement = test_connection()->pdo()->prepare(
        'SELECT * FROM activity_log WHERE action = :action ORDER BY created_at DESC, id DESC LIMIT 1',
    );
    $statement->execute(['action' => $action]);

    return $statement->fetch();
}

it('translates ClientCreated into a client.created row', function () {
    $userId = create_test_user(email: 'listener-client-created@test.com');
    $clientId = create_test_client(email: 'listener-client-created-subject@test.com');

    make_activity_log_listener()->onClientCreated(new ClientCreated($clientId, 'Acme Co', $userId));

    $row = latest_activity_log_row('client.created');
    expect($row['description'])->toBe('Created client Acme Co')
        ->and($row['subject_type'])->toBe('client')
        ->and($row['subject_id'])->toBe($clientId)
        ->and($row['user_id'])->toBe($userId);
});

it('translates ClientUpdated into a client.updated row', function () {
    $clientId = create_test_client(email: 'listener-client-updated@test.com');

    make_activity_log_listener()->onClientUpdated(new ClientUpdated($clientId, 'Acme Co', null));

    $row = latest_activity_log_row('client.updated');
    expect($row['description'])->toBe('Updated client Acme Co')
        ->and($row['subject_type'])->toBe('client')
        ->and($row['subject_id'])->toBe($clientId);
});

it('translates ClientDeleted into a client.deleted row', function () {
    $clientId = create_test_client(email: 'listener-client-deleted@test.com');

    make_activity_log_listener()->onClientDeleted(new ClientDeleted($clientId, 'Acme Co', null));

    $row = latest_activity_log_row('client.deleted');
    expect($row['description'])->toBe('Deleted client Acme Co')
        ->and($row['subject_type'])->toBe('client')
        ->and($row['subject_id'])->toBe($clientId);
});

it('translates ProjectCreated into a project.created row', function () {
    $projectId = create_test_project(name: 'Acme Site');

    make_activity_log_listener()->onProjectCreated(new ProjectCreated($projectId, 'Acme Site', null));

    $row = latest_activity_log_row('project.created');
    expect($row['description'])->toBe('Created project Acme Site')
        ->and($row['subject_type'])->toBe('project')
        ->and($row['subject_id'])->toBe($projectId);
});

it('translates ProjectUpdated into a project.updated row', function () {
    $projectId = create_test_project(name: 'Acme Site');

    make_activity_log_listener()->onProjectUpdated(new ProjectUpdated($projectId, 'Acme Site', null));

    $row = latest_activity_log_row('project.updated');
    expect($row['description'])->toBe('Updated project Acme Site')
        ->and($row['subject_type'])->toBe('project')
        ->and($row['subject_id'])->toBe($projectId);
});

it('translates ProjectDeleted into a project.deleted row', function () {
    $projectId = create_test_project(name: 'Acme Site');

    make_activity_log_listener()->onProjectDeleted(new ProjectDeleted($projectId, 'Acme Site', null));

    $row = latest_activity_log_row('project.deleted');
    expect($row['description'])->toBe('Deleted project Acme Site')
        ->and($row['subject_type'])->toBe('project')
        ->and($row['subject_id'])->toBe($projectId);
});

it('translates UserLoggedIn into an auth.login row with no subject', function () {
    $userId = create_test_user(email: 'listener-login@test.com', name: 'Dana Lee');

    make_activity_log_listener()->onUserLoggedIn(new UserLoggedIn($userId, 'Dana Lee'));

    $row = latest_activity_log_row('auth.login');
    expect($row['description'])->toBe('Dana Lee logged in')
        ->and($row['user_id'])->toBe($userId)
        ->and($row['subject_type'])->toBeNull()
        ->and($row['subject_id'])->toBeNull();
});

it('translates PasswordWasReset into an auth.password_reset row', function () {
    $userId = create_test_user(email: 'listener-reset@test.com');

    make_activity_log_listener()->onPasswordWasReset(new PasswordWasReset($userId));

    $row = latest_activity_log_row('auth.password_reset');
    expect($row['description'])->toBe('Password was reset')
        ->and($row['user_id'])->toBe($userId);
});
