import { jsPDF } from 'jspdf'
import autoTable from 'jspdf-autotable'
import type { InvoiceDetail } from './api/invoices'

export interface InvoicePdfIssuer {
  name: string
  email: string
}

function formatCurrency(value: number): string {
  return value.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

function formatDiscount(type: InvoiceDetail['discountType'], value: number): string {
  if (type === 'percent') return `${value}%`
  if (type === 'fixed') return formatCurrency(value)
  return '-'
}

// jspdf-autotable augments the jsPDF instance with `lastAutoTable` at
// runtime (see its README), but doesn't declare it on jsPDF's own type -
// this is the documented way to read back where the table ended.
interface WithLastAutoTable {
  lastAutoTable?: { finalY: number }
}

// Entirely client-side (jsPDF + jspdf-autotable) - no backend involvement,
// so nothing here depends on a PHP PDF library or its version/extension
// constraints. Renders straight from the same InvoiceDetail the page
// already has plus the logged-in user (the issuer - see
// invoices.$invoiceId.tsx for where that comes from), and triggers a
// browser download.
export function downloadInvoicePdf(invoice: InvoiceDetail, issuer: InvoicePdfIssuer): void {
  const doc = new jsPDF({ unit: 'pt', format: 'a4' })
  const pageWidth = doc.internal.pageSize.getWidth()
  const marginX = 40
  let y = 50

  doc.setFont('helvetica', 'bold')
  doc.setFontSize(20)
  doc.text('INVOICE', marginX, y)
  doc.setFont('helvetica', 'normal')
  doc.setFontSize(11)
  doc.text(invoice.invoiceNumber, pageWidth - marginX, y, { align: 'right' })

  y += 34
  doc.setFontSize(9)
  doc.setTextColor(120)
  doc.text('FROM', marginX, y)
  doc.text('BILL TO', 300, y)

  y += 14
  doc.setFontSize(10)
  doc.setTextColor(20)
  doc.text(issuer.name, marginX, y)
  doc.text(invoice.clientName, 300, y)

  y += 14
  doc.setTextColor(90)
  doc.text(issuer.email, marginX, y)
  doc.text(invoice.projectName, 300, y)

  y += 30
  const meta: [string, string][] = [
    ['Status', invoice.status],
    ...(invoice.issueDate ? ([['Issued', invoice.issueDate]] as [string, string][]) : []),
    ...(invoice.dueDate ? ([['Due', invoice.dueDate]] as [string, string][]) : []),
    ...(invoice.paidAt ? ([['Paid', invoice.paidAt]] as [string, string][]) : []),
  ]
  doc.setFontSize(9)
  for (const [label, value] of meta) {
    doc.setTextColor(120)
    doc.text(`${label}:`, marginX, y)
    doc.setTextColor(20)
    doc.text(value, marginX + 50, y)
    y += 14
  }

  y += 10
  autoTable(doc, {
    startY: y,
    margin: { left: marginX, right: marginX },
    head: [['Description', 'Qty', 'Unit price', 'Discount', 'Total']],
    body: invoice.lines.map((line) => [
      line.description,
      String(line.quantity),
      formatCurrency(line.unitPrice),
      formatDiscount(line.discountType, line.discountValue),
      formatCurrency(line.lineTotal),
    ]),
    styles: { fontSize: 9, cellPadding: 6, textColor: 20 },
    headStyles: { fillColor: [24, 24, 27], textColor: 255 },
    columnStyles: {
      1: { halign: 'right' },
      2: { halign: 'right' },
      3: { halign: 'right' },
      4: { halign: 'right' },
    },
  })

  y = ((doc as unknown as WithLastAutoTable).lastAutoTable?.finalY ?? y) + 24

  const totalsLabelX = pageWidth - marginX - 100
  doc.setFontSize(10)
  doc.setTextColor(120)
  doc.text('Subtotal', totalsLabelX, y)
  doc.setTextColor(20)
  doc.text(formatCurrency(invoice.subtotal), pageWidth - marginX, y, { align: 'right' })

  if (invoice.discount > 0) {
    y += 16
    doc.setTextColor(120)
    doc.text('Discount', totalsLabelX, y)
    doc.setTextColor(20)
    doc.text(`-${formatCurrency(invoice.discount)}`, pageWidth - marginX, y, { align: 'right' })
  }

  y += 20
  doc.setFont('helvetica', 'bold')
  doc.setFontSize(12)
  doc.text('Total', totalsLabelX, y)
  doc.text(formatCurrency(invoice.total), pageWidth - marginX, y, { align: 'right' })
  doc.setFont('helvetica', 'normal')

  if (invoice.notes) {
    y += 36
    doc.setFontSize(9)
    doc.setTextColor(120)
    doc.text('Notes', marginX, y)
    y += 14
    doc.setTextColor(20)
    const wrapped = doc.splitTextToSize(invoice.notes, pageWidth - marginX * 2) as string[]
    doc.text(wrapped, marginX, y)
  }

  doc.save(`${invoice.invoiceNumber}.pdf`)
}
