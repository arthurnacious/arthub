<?php

declare(strict_types=1);

namespace App\Modules\Activity;

use App\Modules\Activity\Listeners\ActivityLogListener;
use Hyperdrive\Application\Module;

#[Module(controllers: [ActivityController::class], listeners: [ActivityLogListener::class])]
class ActivityModule
{
}
