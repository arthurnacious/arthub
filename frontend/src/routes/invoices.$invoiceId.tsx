import { useEffect, useState } from 'react'
import { Link, createFileRoute } from '@tanstack/react-router'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Panel } from '../components/Panel'
import { StatusBadge } from '../components/StatusBadge'
import { InvoiceForm } from '../components/InvoiceForm'
import { DetailFieldsSkeleton } from '../components/DetailFieldsSkeleton'
import { ErrorState } from '../components/ErrorState'
import { Skeleton } from '../components/Skeleton'
import { TableRowsSkeleton } from '../components/TableRowsSkeleton'
import { INVOICE_STATUS_TONE, deleteInvoice, fetchInvoice, updateInvoice } from '../lib/api/invoices'
import type { InvoiceDetail, InvoiceInput } from '../lib/api/invoices'
import { useConfirm } from '../hooks/use-confirm'

const LINE_ITEM_SKELETON_COLUMNS = [
  { className: 'py-2 pr-2', width: 'w-40' },
  { className: 'py-2 pr-2 text-right', width: 'w-6', align: 'right' as const },
  { className: 'py-2 pr-2 text-right', width: 'w-14', align: 'right' as const },
  { className: 'py-2 pr-2 text-right', width: 'w-10', align: 'right' as const },
  { className: 'py-2 text-right', width: 'w-14', align: 'right' as const },
]

interface InvoiceDetailSearch {
  edit: boolean
}

export const Route = createFileRoute('/invoices/$invoiceId')({
  validateSearch: (search: Record<string, unknown>): InvoiceDetailSearch => ({
    edit: search.edit === true || search.edit === 'true',
  }),
  component: InvoiceDetailPage,
})

const BUTTON_CLASSES =
  'rounded border border-[var(--border)] px-3 py-1.5 text-xs text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]'

function formatCurrency(value: number): string {
  return value.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

function formatDiscount(type: InvoiceDetail['discountType'], value: number): string {
  if (type === 'percent') return `${value}%`
  if (type === 'fixed') return formatCurrency(value)
  return '-'
}

function toFormValues(invoice: InvoiceDetail): InvoiceInput {
  return {
    projectId: invoice.projectId,
    quoteId: invoice.quoteId,
    status: invoice.status,
    issueDate: invoice.issueDate,
    dueDate: invoice.dueDate,
    discountType: invoice.discountType,
    discountValue: invoice.discountValue,
    notes: invoice.notes,
    lineItems: invoice.lines.map((line) => ({
      description: line.description,
      quantity: line.quantity,
      unitPrice: line.unitPrice,
      discountType: line.discountType,
      discountValue: line.discountValue,
    })),
  }
}

function InvoiceDetailPage() {
  const { invoiceId } = Route.useParams()
  const { edit: editFromUrl } = Route.useSearch()
  const navigate = Route.useNavigate()
  const queryClient = useQueryClient()
  const { confirm, dialog } = useConfirm()

  const { data: invoice, isPending, isError, refetch } = useQuery({
    queryKey: ['invoice', invoiceId],
    queryFn: () => fetchInvoice(invoiceId),
  })

  const [editing, setEditing] = useState(editFromUrl)
  const [form, setForm] = useState<InvoiceInput | null>(null)

  useEffect(() => {
    if (invoice) setForm(toFormValues(invoice))
  }, [invoice])

  const updateMutation = useMutation({
    mutationFn: (values: InvoiceInput) => updateInvoice(invoiceId, values),
    onSuccess: (updated) => {
      queryClient.setQueryData(['invoice', invoiceId], updated)
      queryClient.invalidateQueries({ queryKey: ['invoices'] })
      setEditing(false)
    },
  })

  const deleteMutation = useMutation({
    mutationFn: () => deleteInvoice(invoiceId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['invoices'] })
      navigate({ to: '/invoices', search: { page: 1, q: '', status: '' } })
    },
  })

  async function handleDelete() {
    if (!invoice) return
    const confirmed = await confirm({
      title: 'delete invoice',
      message: `Delete ${invoice.invoiceNumber}? This can't be undone.`,
      confirmLabel: 'delete',
      tone: 'danger',
    })
    if (!confirmed) return
    deleteMutation.mutate()
  }

  if (isPending) {
    return (
      <div>
        <Skeleton className="h-6 w-32" />
        <div className="mt-6 max-w-2xl">
          <Panel title="invoice">
            <div className="flex flex-col gap-4">
              <DetailFieldsSkeleton rows={3} />
              <table className="w-full text-left text-xs">
                <thead>
                  <tr className="border-b border-[var(--border)] uppercase tracking-wider text-[var(--text-dim)]">
                    <th className="py-2 pr-2 font-normal">Description</th>
                    <th className="py-2 pr-2 text-right font-normal">Qty</th>
                    <th className="py-2 pr-2 text-right font-normal">Unit price</th>
                    <th className="py-2 pr-2 text-right font-normal">Discount</th>
                    <th className="py-2 text-right font-normal">Total</th>
                  </tr>
                </thead>
                <tbody>
                  <TableRowsSkeleton rows={3} columns={LINE_ITEM_SKELETON_COLUMNS} />
                </tbody>
              </table>
            </div>
          </Panel>
        </div>
      </div>
    )
  }

  if (isError) {
    return (
      <div>
        <div className="flex items-center justify-between">
          <h1 className="glow-text text-lg font-bold text-[var(--accent)]">INVOICE</h1>
          <Link
            to="/invoices"
            search={{ page: 1, q: '', status: '' }}
            className="text-xs text-[var(--text-dim)] hover:text-[var(--accent)]"
          >
            &larr; back to invoices
          </Link>
        </div>
        <div className="mt-6">
          <ErrorState
            title="COULDN'T LOAD INVOICE"
            message="Something went wrong fetching this invoice."
            onRetry={() => refetch()}
          />
        </div>
      </div>
    )
  }

  if (!invoice || !form) {
    return (
      <div>
        <h1 className="glow-text text-lg font-bold text-[var(--accent)]">INVOICE NOT FOUND</h1>
        <Link
          to="/invoices"
          search={{ page: 1, q: '', status: '' }}
          className="mt-4 inline-block text-sm text-[var(--accent)] hover:underline"
        >
          &larr; back to invoices
        </Link>
      </div>
    )
  }

  return (
    <div>
      <div className="flex items-center justify-between">
        <h1 className="glow-text text-lg font-bold text-[var(--accent)]">{invoice.invoiceNumber}</h1>
        <Link
          to="/invoices"
          search={{ page: 1, q: '', status: '' }}
          className="text-xs text-[var(--text-dim)] hover:text-[var(--accent)]"
        >
          &larr; back to invoices
        </Link>
      </div>

      <div className="mt-6 max-w-2xl">
        <Panel title={editing ? 'edit invoice' : 'invoice'}>
          {editing ? (
            <InvoiceForm
              values={form}
              onChange={setForm}
              onSubmit={() => updateMutation.mutate(form)}
              onCancel={() => {
                setEditing(false)
                setForm(toFormValues(invoice))
              }}
              submitLabel="save"
              submitting={updateMutation.isPending}
            />
          ) : (
            <div className="flex flex-col gap-4 text-sm">
              <div className="flex flex-wrap items-center gap-x-6 gap-y-2">
                <div>
                  <span className="text-[var(--text-dim)]">project: </span>
                  {invoice.projectName}
                </div>
                <div>
                  <span className="text-[var(--text-dim)]">client: </span>
                  {invoice.clientName}
                </div>
                <div>
                  <span className="text-[var(--text-dim)]">status: </span>
                  <StatusBadge label={invoice.status} tone={INVOICE_STATUS_TONE[invoice.status]} />
                </div>
                {invoice.quoteNumber && (
                  <div>
                    <span className="text-[var(--text-dim)]">from quote: </span>
                    <Link
                      to="/quotes/$quoteId"
                      params={{ quoteId: invoice.quoteId! }}
                      search={{ edit: false }}
                      className="text-[var(--accent)] hover:underline"
                    >
                      {invoice.quoteNumber}
                    </Link>
                  </div>
                )}
                {invoice.issueDate && (
                  <div>
                    <span className="text-[var(--text-dim)]">issued: </span>
                    {invoice.issueDate}
                  </div>
                )}
                {invoice.dueDate && (
                  <div>
                    <span className="text-[var(--text-dim)]">due: </span>
                    {invoice.dueDate}
                  </div>
                )}
                {invoice.paidAt && (
                  <div>
                    <span className="text-[var(--text-dim)]">paid: </span>
                    {invoice.paidAt}
                  </div>
                )}
              </div>

              {invoice.notes && (
                <div>
                  <span className="text-[var(--text-dim)]">notes: </span>
                  {invoice.notes}
                </div>
              )}

              <table className="w-full text-left text-xs">
                <thead>
                  <tr className="border-b border-[var(--border)] uppercase tracking-wider text-[var(--text-dim)]">
                    <th className="py-2 pr-2 font-normal">Description</th>
                    <th className="py-2 pr-2 text-right font-normal">Qty</th>
                    <th className="py-2 pr-2 text-right font-normal">Unit price</th>
                    <th className="py-2 pr-2 text-right font-normal">Discount</th>
                    <th className="py-2 text-right font-normal">Total</th>
                  </tr>
                </thead>
                <tbody>
                  {invoice.lines.length === 0 ? (
                    <tr>
                      <td colSpan={5} className="py-3 text-center text-[var(--text-dim)]">
                        -- no line items --
                      </td>
                    </tr>
                  ) : (
                    invoice.lines.map((line) => (
                      <tr key={line.id} className="border-b border-[var(--border)]">
                        <td className="py-2 pr-2 text-[var(--text)]">{line.description}</td>
                        <td className="py-2 pr-2 text-right text-[var(--text-dim)]">{line.quantity}</td>
                        <td className="py-2 pr-2 text-right text-[var(--text-dim)]">
                          {formatCurrency(line.unitPrice)}
                        </td>
                        <td className="py-2 pr-2 text-right text-[var(--text-dim)]">
                          {formatDiscount(line.discountType, line.discountValue)}
                        </td>
                        <td className="py-2 text-right text-[var(--text)]">{formatCurrency(line.lineTotal)}</td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>

              <div className="self-end text-right text-xs text-[var(--text-dim)]">
                <div>subtotal: {formatCurrency(invoice.subtotal)}</div>
                {invoice.discount > 0 && <div>discount: -{formatCurrency(invoice.discount)}</div>}
                <div className="text-sm font-bold text-[var(--text)]">
                  total: {formatCurrency(invoice.total)}
                </div>
              </div>

              <div className="mt-2 flex gap-2">
                <button type="button" onClick={() => setEditing(true)} className={BUTTON_CLASSES}>
                  edit
                </button>
                <button
                  type="button"
                  onClick={handleDelete}
                  className={`${BUTTON_CLASSES} hover:border-red-400 hover:text-red-400`}
                >
                  delete
                </button>
              </div>
            </div>
          )}
        </Panel>
      </div>
      {dialog}
    </div>
  )
}
