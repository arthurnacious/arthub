import { useEffect, useState } from 'react'
import { Link, createFileRoute } from '@tanstack/react-router'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Panel } from '../components/Panel'
import { StatusBadge } from '../components/StatusBadge'
import { QuoteForm } from '../components/QuoteForm'
import { QUOTE_STATUS_TONE, deleteQuote, fetchQuote, updateQuote } from '../lib/api/quotes'
import type { QuoteDetail, QuoteInput } from '../lib/api/quotes'
import { useConfirm } from '../hooks/use-confirm'

interface QuoteDetailSearch {
  edit: boolean
}

export const Route = createFileRoute('/quotes/$quoteId')({
  validateSearch: (search: Record<string, unknown>): QuoteDetailSearch => ({
    edit: search.edit === true || search.edit === 'true',
  }),
  component: QuoteDetailPage,
})

const BUTTON_CLASSES =
  'rounded border border-[var(--border)] px-3 py-1.5 text-xs text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]'

function formatCurrency(value: number): string {
  return value.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })
}

function formatDiscount(type: QuoteDetail['discountType'], value: number): string {
  if (type === 'percent') return `${value}%`
  if (type === 'fixed') return formatCurrency(value)
  return '-'
}

function toFormValues(quote: QuoteDetail): QuoteInput {
  return {
    projectId: quote.projectId,
    status: quote.status,
    issueDate: quote.issueDate,
    expiryDate: quote.expiryDate,
    discountType: quote.discountType,
    discountValue: quote.discountValue,
    notes: quote.notes,
    lineItems: quote.lines.map((line) => ({
      description: line.description,
      quantity: line.quantity,
      unitPrice: line.unitPrice,
      discountType: line.discountType,
      discountValue: line.discountValue,
    })),
  }
}

function QuoteDetailPage() {
  const { quoteId } = Route.useParams()
  const { edit: editFromUrl } = Route.useSearch()
  const navigate = Route.useNavigate()
  const queryClient = useQueryClient()
  const { confirm, dialog } = useConfirm()

  const { data: quote, isPending, isError } = useQuery({
    queryKey: ['quote', quoteId],
    queryFn: () => fetchQuote(quoteId),
  })

  const [editing, setEditing] = useState(editFromUrl)
  const [form, setForm] = useState<QuoteInput | null>(null)

  useEffect(() => {
    if (quote) setForm(toFormValues(quote))
  }, [quote])

  const updateMutation = useMutation({
    mutationFn: (values: QuoteInput) => updateQuote(quoteId, values),
    onSuccess: (updated) => {
      queryClient.setQueryData(['quote', quoteId], updated)
      queryClient.invalidateQueries({ queryKey: ['quotes'] })
      setEditing(false)
    },
  })

  const deleteMutation = useMutation({
    mutationFn: () => deleteQuote(quoteId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['quotes'] })
      navigate({ to: '/quotes', search: { page: 1, q: '', status: '' } })
    },
  })

  async function handleDelete() {
    if (!quote) return
    const confirmed = await confirm({
      title: 'delete quote',
      message: `Delete ${quote.quoteNumber}? This can't be undone.`,
      confirmLabel: 'delete',
      tone: 'danger',
    })
    if (!confirmed) return
    deleteMutation.mutate()
  }

  if (isPending) {
    return <p className="text-sm text-[var(--text-dim)]">loading...</p>
  }

  if (isError) {
    return (
      <div>
        <h1 className="glow-text text-lg font-bold text-[var(--accent)]">COULDN'T LOAD QUOTE</h1>
        <p className="mt-2 text-sm text-red-400">
          Something went wrong fetching this quote. Try refreshing the page.
        </p>
        <Link
          to="/quotes"
          search={{ page: 1, q: '', status: '' }}
          className="mt-4 inline-block text-sm text-[var(--accent)] hover:underline"
        >
          &larr; back to quotes
        </Link>
      </div>
    )
  }

  if (!quote || !form) {
    return (
      <div>
        <h1 className="glow-text text-lg font-bold text-[var(--accent)]">QUOTE NOT FOUND</h1>
        <Link
          to="/quotes"
          search={{ page: 1, q: '', status: '' }}
          className="mt-4 inline-block text-sm text-[var(--accent)] hover:underline"
        >
          &larr; back to quotes
        </Link>
      </div>
    )
  }

  return (
    <div>
      <div className="flex items-center justify-between">
        <h1 className="glow-text text-lg font-bold text-[var(--accent)]">{quote.quoteNumber}</h1>
        <Link
          to="/quotes"
          search={{ page: 1, q: '', status: '' }}
          className="text-xs text-[var(--text-dim)] hover:text-[var(--accent)]"
        >
          &larr; back to quotes
        </Link>
      </div>

      <div className="mt-6 max-w-2xl">
        <Panel title={editing ? 'edit quote' : 'quote'}>
          {editing ? (
            <QuoteForm
              values={form}
              onChange={setForm}
              onSubmit={() => updateMutation.mutate(form)}
              onCancel={() => {
                setEditing(false)
                setForm(toFormValues(quote))
              }}
              submitLabel="save"
              submitting={updateMutation.isPending}
            />
          ) : (
            <div className="flex flex-col gap-4 text-sm">
              <div className="flex flex-wrap items-center gap-x-6 gap-y-2">
                <div>
                  <span className="text-[var(--text-dim)]">project: </span>
                  {quote.projectName}
                </div>
                <div>
                  <span className="text-[var(--text-dim)]">client: </span>
                  {quote.clientName}
                </div>
                <div>
                  <span className="text-[var(--text-dim)]">status: </span>
                  <StatusBadge label={quote.status} tone={QUOTE_STATUS_TONE[quote.status]} />
                </div>
                {quote.issueDate && (
                  <div>
                    <span className="text-[var(--text-dim)]">issued: </span>
                    {quote.issueDate}
                  </div>
                )}
                {quote.expiryDate && (
                  <div>
                    <span className="text-[var(--text-dim)]">expires: </span>
                    {quote.expiryDate}
                  </div>
                )}
              </div>

              {quote.notes && (
                <div>
                  <span className="text-[var(--text-dim)]">notes: </span>
                  {quote.notes}
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
                  {quote.lines.length === 0 ? (
                    <tr>
                      <td colSpan={5} className="py-3 text-center text-[var(--text-dim)]">
                        -- no line items --
                      </td>
                    </tr>
                  ) : (
                    quote.lines.map((line) => (
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
                <div>subtotal: {formatCurrency(quote.subtotal)}</div>
                {quote.discount > 0 && <div>discount: -{formatCurrency(quote.discount)}</div>}
                <div className="text-sm font-bold text-[var(--text)]">
                  total: {formatCurrency(quote.total)}
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
