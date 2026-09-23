import { describe, expect, it, vi } from 'vitest'
import { jsPDF } from 'jspdf'
import { downloadInvoicePdf } from './invoice-pdf'
import type { InvoiceDetail } from './api/invoices'

function makeInvoice(overrides: Partial<InvoiceDetail> = {}): InvoiceDetail {
  return {
    id: 'inv-1',
    invoiceNumber: 'INV-0001',
    projectId: 'proj-1',
    projectName: 'Website revamp',
    clientId: 'client-1',
    clientName: 'Acme Co',
    quoteId: null,
    quoteNumber: null,
    status: 'sent',
    issueDate: '2026-09-01',
    dueDate: '2026-09-15',
    paidAt: null,
    discountType: 'percent',
    discountValue: 10,
    notes: 'Thanks for your business.',
    subtotal: 1000,
    discount: 100,
    total: 900,
    lines: [
      {
        id: 'line-1',
        description: 'Design work',
        quantity: 10,
        unitPrice: 100,
        discountType: null,
        discountValue: 0,
        lineTotal: 1000,
        position: 0,
      },
    ],
    ...overrides,
  }
}

// jsPDF does real PDF-document work (not DOM/canvas rendering), so it runs
// fine under jsdom. `save` is assigned per-instance from the static
// jsPDF.API namespace at construction time (not on jsPDF.prototype), and
// its real implementation is environment-specific (writes to disk under
// Node, triggers a browser download in a real browser) - so it's stubbed
// at the jsPDF.API level, read fresh by every `new jsPDF()` call, same as
// any other side-effecting API in these tests.
describe('downloadInvoicePdf', () => {
  it('generates a PDF and saves it as "<invoiceNumber>.pdf"', () => {
    const saveSpy = vi.fn()
    const original = jsPDF.API.save
    jsPDF.API.save = saveSpy

    downloadInvoicePdf(makeInvoice(), { name: 'Arthur', email: 'arthur@example.com' })

    expect(saveSpy).toHaveBeenCalledWith('INV-0001.pdf')

    jsPDF.API.save = original
  })

  it('does not throw when there are no line items, discount or notes', () => {
    const original = jsPDF.API.save
    jsPDF.API.save = vi.fn()

    expect(() =>
      downloadInvoicePdf(
        makeInvoice({ lines: [], discount: 0, notes: '', issueDate: null, dueDate: null, paidAt: '2026-09-10' }),
        { name: 'Arthur', email: 'arthur@example.com' },
      ),
    ).not.toThrow()

    jsPDF.API.save = original
  })
})
