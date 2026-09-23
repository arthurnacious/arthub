<?php

declare(strict_types=1);

namespace App\Modules\Quotes\Dto;

// Identical shape to CreateQuoteDto - the frontend always sends the full
// record (including the full lineItems array) on update, same as Projects.
class UpdateQuoteDto extends CreateQuoteDto
{
}
