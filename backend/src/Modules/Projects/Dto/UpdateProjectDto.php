<?php

declare(strict_types=1);

namespace App\Modules\Projects\Dto;

// Same shape as create - the frontend always sends the full project
// record on update, not a sparse patch.
class UpdateProjectDto extends CreateProjectDto
{
}
