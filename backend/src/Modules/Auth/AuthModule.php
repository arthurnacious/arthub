<?php

declare(strict_types=1);

namespace App\Modules\Auth;

use App\Modules\Auth\Services\AuthService;
use Hyperdrive\Application\Module;

#[Module(
    controllers: [AuthController::class],
    injectables: [AuthService::class],
    exports: [AuthService::class],
)]
class AuthModule
{
}
