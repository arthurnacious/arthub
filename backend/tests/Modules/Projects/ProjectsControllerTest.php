<?php

declare(strict_types=1);

use App\Modules\Projects\Dto\CreateProjectDto;
use App\Modules\Projects\Dto\UpdateProjectDto;
use App\Modules\Projects\ProjectsController;
use Hyperdrive\Http\Request;

function make_projects_controller(): ProjectsController
{
    return new ProjectsController(test_connection(), make_event_dispatcher());
}

function make_test_upload(string $name, string $mime, string $contents = 'fake-image-bytes'): array
{
    $tmp = tempnam(sys_get_temp_dir(), 'upload');
    file_put_contents($tmp, $contents);

    return [
        'name' => $name,
        'type' => $mime,
        'tmp_name' => $tmp,
        'error' => UPLOAD_ERR_OK,
        'size' => strlen($contents),
    ];
}

afterEach(function () {
    $storageRoot = dirname(__DIR__, 3) . '/storage/app/projects';
    if (is_dir($storageRoot)) {
        $iterator = new RecursiveIteratorIterator(
            new RecursiveDirectoryIterator($storageRoot, FilesystemIterator::SKIP_DOTS),
            RecursiveIteratorIterator::CHILD_FIRST,
        );
        foreach ($iterator as $item) {
            $item->isDir() ? rmdir($item->getPathname()) : unlink($item->getPathname());
        }
    }
});

it('lists projects newest first, with pagination metadata', function () {
    create_test_project(name: 'Alpha');
    create_test_project(name: 'Beta');

    $response = make_projects_controller()->index(new Request());
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['data'])->toHaveCount(2)
        ->and($body['data'][0]['name'])->toBe('Beta')
        ->and($body['total'])->toBe(2)
        ->and($body['page'])->toBe(1)
        ->and($body['pageSize'])->toBe(10);
});

it('paginates', function () {
    create_test_project(name: 'A');
    create_test_project(name: 'B');
    create_test_project(name: 'C');

    $response = make_projects_controller()->index(new Request(query: ['page' => '2', 'pageSize' => '2']));
    $body = json_decode($response->getContent(), true);

    expect($body['data'])->toHaveCount(1)
        ->and($body['total'])->toBe(3)
        ->and($body['page'])->toBe(2);
});

it('searches by name or description', function () {
    create_test_project(name: 'Nimbus Website');
    create_test_project(name: 'Other Thing');

    $response = make_projects_controller()->index(new Request(query: ['q' => 'nimbus']));
    $body = json_decode($response->getContent(), true);

    expect($body['data'])->toHaveCount(1)
        ->and($body['data'][0]['name'])->toBe('Nimbus Website');
});

it('filters by category', function () {
    create_test_project(name: 'Logo Work', category: 'branding');
    create_test_project(name: 'Site Work', category: 'web');

    $response = make_projects_controller()->index(new Request(query: ['category' => 'branding']));
    $body = json_decode($response->getContent(), true);

    expect($body['data'])->toHaveCount(1)
        ->and($body['data'][0]['name'])->toBe('Logo Work');
});

it('lists distinct categories', function () {
    create_test_project(category: 'branding');
    create_test_project(category: 'branding');
    create_test_project(category: 'web');

    $response = make_projects_controller()->categories();
    $body = json_decode($response->getContent(), true);

    expect($body['data'])->toBe(['branding', 'web']);
});

it('shows a single project with its client name and empty images', function () {
    $clientId = create_test_client(company: 'Solo Co', email: 'solo-project@test.com');
    $id = create_test_project(clientId: $clientId, name: 'Solo Project');

    $response = make_projects_controller()->show($id);
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['name'])->toBe('Solo Project')
        ->and($body['clientName'])->toBe('Solo Co')
        ->and($body['images'])->toBe([]);
});

it('404s for an unknown project id', function () {
    $response = make_projects_controller()->show('00000000-0000-0000-0000-000000000000');

    expect($response->getStatusCode())->toBe(404);
});

it('creates a project and logs the activity', function () {
    $userId = create_test_user(email: 'creator@test.com');
    $clientId = create_test_client(email: 'client-for-create@test.com');
    $dto = new CreateProjectDto([
        'clientId' => $clientId,
        'name' => 'New Project',
        'description' => 'A thing we built',
        'category' => 'web',
        'url' => 'https://example.com',
        'status' => 'active',
    ]);

    $response = make_projects_controller()->store(
        $dto,
        new Request(attributes: ['auth_user' => ['sub' => $userId]]),
    );
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(201)
        ->and($body['name'])->toBe('New Project')
        ->and($body['url'])->toBe('https://example.com')
        ->and($body['category'])->toBe('web');

    $log = test_connection()->pdo()
        ->query("SELECT * FROM activity_log WHERE action = 'project.created'")
        ->fetch();
    expect($log['description'])->toBe('Created project New Project')
        ->and($log['user_id'])->toBe($userId);
});

it('rejects an invalid status on create', function () {
    $clientId = create_test_client(email: 'client-invalid-status@test.com');
    $dto = new CreateProjectDto([
        'clientId' => $clientId,
        'name' => 'New Project',
        'category' => 'web',
        'status' => 'bogus',
    ]);

    $response = make_projects_controller()->store($dto, new Request());

    expect($response->getStatusCode())->toBe(422);
});

it('rejects creating a project for an unknown client', function () {
    $dto = new CreateProjectDto([
        'clientId' => '00000000-0000-0000-0000-000000000000',
        'name' => 'New Project',
        'category' => 'web',
        'status' => 'active',
    ]);

    $response = make_projects_controller()->store($dto, new Request());

    expect($response->getStatusCode())->toBe(422);
});

it('updates a project', function () {
    $id = create_test_project(name: 'Old Name');
    $clientId = create_test_client(email: 'client-for-update@test.com');
    $dto = new UpdateProjectDto([
        'clientId' => $clientId,
        'name' => 'New Name',
        'category' => 'branding',
        'status' => 'completed',
    ]);

    $response = make_projects_controller()->update($id, $dto, new Request());
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(200)
        ->and($body['name'])->toBe('New Name')
        ->and($body['status'])->toBe('completed');
});

it('404s updating an unknown project', function () {
    $clientId = create_test_client(email: 'client-404-update@test.com');
    $dto = new UpdateProjectDto([
        'clientId' => $clientId,
        'name' => 'X',
        'category' => 'web',
        'status' => 'active',
    ]);

    $response = make_projects_controller()->update(
        '00000000-0000-0000-0000-000000000000',
        $dto,
        new Request(),
    );

    expect($response->getStatusCode())->toBe(404);
});

it('rejects an invalid status on update', function () {
    $id = create_test_project();
    $clientId = create_test_client(email: 'client-invalid-update@test.com');
    $dto = new UpdateProjectDto([
        'clientId' => $clientId,
        'name' => 'X',
        'category' => 'web',
        'status' => 'bogus',
    ]);

    $response = make_projects_controller()->update($id, $dto, new Request());

    expect($response->getStatusCode())->toBe(422);
});

it('deletes a project, logs the activity, and cleans up its image files', function () {
    $userId = create_test_user(email: 'deleter@test.com');
    $id = create_test_project(name: 'Doomed Project');

    $upload = make_test_upload('a.jpg', 'image/jpeg');
    make_projects_controller()->uploadImages($id, new Request(files: ['images' => $upload]));

    $pdo = test_connection()->pdo();
    $path = $pdo->query("SELECT path FROM project_images WHERE project_id = '{$id}'")->fetchColumn();
    $fullPath = dirname(__DIR__, 3) . "/storage/app/{$path}";
    expect(is_file($fullPath))->toBeTrue();

    $response = make_projects_controller()->destroy(
        $id,
        new Request(attributes: ['auth_user' => ['sub' => $userId]]),
    );

    expect($response->getStatusCode())->toBe(200)
        ->and(is_file($fullPath))->toBeFalse();

    $log = test_connection()->pdo()
        ->query("SELECT * FROM activity_log WHERE action = 'project.deleted'")
        ->fetch();
    expect($log['description'])->toBe('Deleted project Doomed Project');
});

it('404s deleting an unknown project', function () {
    $response = make_projects_controller()->destroy(
        '00000000-0000-0000-0000-000000000000',
        new Request(),
    );

    expect($response->getStatusCode())->toBe(404);
});

it('uploads images for a project', function () {
    $id = create_test_project();
    $upload = make_test_upload('cover.png', 'image/png');

    $response = make_projects_controller()->uploadImages($id, new Request(files: ['images' => $upload]));
    $body = json_decode($response->getContent(), true);

    expect($response->getStatusCode())->toBe(201)
        ->and($body['data'])->toHaveCount(1)
        ->and($body['data'][0]['url'])->toContain('/storage/projects/')
        ->and($body['data'][0]['sortOrder'])->toBe(0);

    $detail = make_projects_controller()->show($id);
    $detailBody = json_decode($detail->getContent(), true);
    expect($detailBody['images'])->toHaveCount(1);
});

it('increments sort order across multiple uploads', function () {
    $id = create_test_project();

    make_projects_controller()->uploadImages(
        $id,
        new Request(files: ['images' => make_test_upload('one.png', 'image/png')]),
    );
    $response = make_projects_controller()->uploadImages(
        $id,
        new Request(files: ['images' => make_test_upload('two.png', 'image/png')]),
    );
    $body = json_decode($response->getContent(), true);

    expect($body['data'][0]['sortOrder'])->toBe(1);
});

it('404s uploading images to an unknown project', function () {
    $upload = make_test_upload('a.jpg', 'image/jpeg');

    $response = make_projects_controller()->uploadImages(
        '00000000-0000-0000-0000-000000000000',
        new Request(files: ['images' => $upload]),
    );

    expect($response->getStatusCode())->toBe(404);
});

it('rejects a non-image upload', function () {
    $id = create_test_project();
    $upload = make_test_upload('virus.exe', 'application/octet-stream');

    $response = make_projects_controller()->uploadImages($id, new Request(files: ['images' => $upload]));

    expect($response->getStatusCode())->toBe(422);
});

it('rejects an oversized upload', function () {
    $id = create_test_project();
    $upload = make_test_upload('huge.png', 'image/png');
    $upload['size'] = 6 * 1024 * 1024;

    $response = make_projects_controller()->uploadImages($id, new Request(files: ['images' => $upload]));

    expect($response->getStatusCode())->toBe(422);
});

it('deletes a single image', function () {
    $id = create_test_project();
    make_projects_controller()->uploadImages(
        $id,
        new Request(files: ['images' => make_test_upload('a.png', 'image/png')]),
    );
    $imageId = test_connection()->pdo()
        ->query("SELECT id FROM project_images WHERE project_id = '{$id}'")
        ->fetchColumn();

    $response = make_projects_controller()->deleteImage($id, $imageId);

    expect($response->getStatusCode())->toBe(200);

    $detail = make_projects_controller()->show($id);
    $detailBody = json_decode($detail->getContent(), true);
    expect($detailBody['images'])->toBe([]);
});

it('404s deleting an unknown image', function () {
    $id = create_test_project();

    $response = make_projects_controller()->deleteImage($id, '00000000-0000-0000-0000-000000000000');

    expect($response->getStatusCode())->toBe(404);
});
