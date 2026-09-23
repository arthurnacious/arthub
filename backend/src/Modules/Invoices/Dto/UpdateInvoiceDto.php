<?php

declare(strict_types=1);

namespace App\Modules\Invoices\Dto;

// Identical shape to CreateInvoiceDto - the frontend always sends the
// full record (including the full lineItems array) on update, same as
// Projects/Quotes.
class UpdateInvoiceDto extends CreateInvoiceDto
{
}
