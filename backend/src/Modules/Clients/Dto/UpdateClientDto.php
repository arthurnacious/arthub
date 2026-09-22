<?php

declare(strict_types=1);

namespace App\Modules\Clients\Dto;

// Same shape as create - the frontend always sends the full client record
// on update, not a sparse patch.
class UpdateClientDto extends CreateClientDto
{
}
