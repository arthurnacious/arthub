<?php

declare(strict_types=1);

use App\Support\ActivityLogger;

function fetch_latest_activity_log_row(): array
{
    $statement = test_connection()->pdo()->query(
        'SELECT * FROM activity_log ORDER BY id DESC LIMIT 1'
    );

    return $statement->fetch();
}

it('writes a row with the given action and description', function () {
    (new ActivityLogger(test_connection()))->log('client.created', 'Acme Ltd was created');

    $row = fetch_latest_activity_log_row();

    expect($row['action'])->toBe('client.created')
        ->and($row['description'])->toBe('Acme Ltd was created')
        ->and($row['user_id'])->toBeNull()
        ->and($row['subject_type'])->toBeNull()
        ->and($row['metadata'])->toBeNull();
});

it('stores the optional user, subject and metadata fields', function () {
    $userId = create_test_user();
    // subject_id is a UUID (every table's id is), but activity_log has no
    // real FK for it (it's polymorphic across subject_type) - any
    // well-formed UUID works for this test, it doesn't need to reference
    // a real invoice.
    $invoiceId = test_connection()->pdo()->query('SELECT uuidv7()')->fetchColumn();

    (new ActivityLogger(test_connection()))->log(
        action: 'invoice.paid',
        description: 'Invoice marked paid',
        userId: $userId,
        subjectType: 'invoice',
        subjectId: $invoiceId,
        metadata: ['amount' => 150.5],
    );

    $row = fetch_latest_activity_log_row();

    expect($row['user_id'])->toBe($userId)
        ->and($row['subject_type'])->toBe('invoice')
        ->and($row['subject_id'])->toBe($invoiceId)
        ->and(json_decode($row['metadata'], true))->toBe(['amount' => 150.5]);
});
