<?php

declare(strict_types=1);

namespace App;

use App\Modules\Activity\ActivityModule;
use App\Modules\Auth\AuthModule;
use App\Modules\Clients\ClientsModule;
use App\Modules\Health\HealthController;
use Hyperdrive\Application\Module;

// Root module. Imports feature modules and registers anything too small
// to warrant its own module yet (the health check).
#[Module(
    imports: [AuthModule::class, ActivityModule::class, ClientsModule::class],
    controllers: [HealthController::class],
)]
class AppModule
{
}
