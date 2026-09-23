import { useEffect, useState } from 'react'
import { createFileRoute } from '@tanstack/react-router'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Panel } from '../components/Panel'
import { InvoicesTable } from '../components/InvoicesTable'
import { Pagination } from '../components/Pagination'
import { Modal } from '../components/Modal'
import { InvoiceForm } from '../components/InvoiceForm'
import { createInvoice, deleteInvoice, fetchInvoices } from '../lib/api/invoices'
import type { Invoice, InvoiceInput, InvoiceStatus } from '../lib/api/invoices'
import { useDebouncedValue } from '../hooks/use-debounced-value'
import { useConfirm } from '../hooks/use-confirm'

const PAGE_SIZE = 10

const STATUS_OPTIONS: InvoiceStatus[] = ['draft', 'sent', 'paid', 'overdue', 'cancelled']

const EMPTY_INVOICE: InvoiceInput = {
  projectId: '',
  quoteId: null,
  status: 'draft',
  issueDate: null,
  dueDate: null,
  discountType: null,
  discountValue: 0,
  notes: '',
  lineItems: [],
}

interface InvoicesSearch {
  page: number
  q: string
  status: string
}

export const Route = createFileRoute('/invoices/')({
  validateSearch: (search: Record<string, unknown>): InvoicesSearch => ({
    page: Number(search.page) > 0 ? Math.floor(Number(search.page)) : 1,
    q: typeof search.q === 'string' ? search.q : '',
    status: typeof search.status === 'string' ? search.status : '',
  }),
  component: InvoicesPage,
})

function InvoicesPage() {
  const { page, q, status } = Route.useSearch()
  const navigate = Route.useNavigate()
  const queryClient = useQueryClient()
  const { confirm, dialog } = useConfirm()

  const [searchInput, setSearchInput] = useState(q)
  const debouncedSearch = useDebouncedValue(searchInput, 300)

  useEffect(() => {
    if (debouncedSearch === q) return
    navigate({
      search: (prev) => ({ ...prev, q: debouncedSearch, page: 1 }),
      replace: true,
    })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debouncedSearch])

  const { data, isPending } = useQuery({
    queryKey: ['invoices', { page, pageSize: PAGE_SIZE, search: q, status }],
    queryFn: () => fetchInvoices({ page, pageSize: PAGE_SIZE, search: q, status }),
    placeholderData: (previousData) => previousData,
  })

  const deleteMutation = useMutation({
    mutationFn: (invoice: Invoice) => deleteInvoice(invoice.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['invoices'] })
    },
  })

  async function handleDelete(invoice: Invoice) {
    const confirmed = await confirm({
      title: 'delete invoice',
      message: `Delete ${invoice.invoiceNumber}? This can't be undone.`,
      confirmLabel: 'delete',
      tone: 'danger',
    })
    if (!confirmed) return
    deleteMutation.mutate(invoice)
  }

  const [newInvoiceOpen, setNewInvoiceOpen] = useState(false)
  const [newInvoiceForm, setNewInvoiceForm] = useState<InvoiceInput>(EMPTY_INVOICE)

  const createMutation = useMutation({
    mutationFn: (values: InvoiceInput) => createInvoice(values),
    onSuccess: (created) => {
      queryClient.invalidateQueries({ queryKey: ['invoices'] })
      setNewInvoiceOpen(false)
      setNewInvoiceForm(EMPTY_INVOICE)
      navigate({ to: '/invoices/$invoiceId', params: { invoiceId: created.id } })
    },
  })

  return (
    <div>
      <div className="flex items-center justify-between">
        <h1 className="glow-text text-lg font-bold text-[var(--accent)]">INVOICES</h1>
        <button
          type="button"
          onClick={() => setNewInvoiceOpen(true)}
          className="rounded border border-[var(--accent)] bg-[var(--accent-dim)] px-3 py-1.5 text-xs text-[var(--accent)] transition-colors hover:bg-[var(--accent)] hover:text-[var(--bg)]"
        >
          + new
        </button>
      </div>
      <p className="mt-2 text-sm text-[var(--text-dim)]">// what actually gets billed and paid</p>

      <div className="mt-6">
        <Panel title="invoices">
          <div className="mb-4 flex flex-col gap-3 sm:flex-row sm:items-center">
            <input
              type="text"
              value={searchInput}
              onChange={(event) => setSearchInput(event.target.value)}
              placeholder="search invoice number, project or client..."
              className="w-full rounded border border-[var(--border)] bg-transparent px-3 py-1.5 text-sm text-[var(--text)] outline-none placeholder:text-[var(--text-dim)] focus:border-[var(--accent)] sm:max-w-xs"
            />
            <select
              value={status}
              onChange={(event) =>
                navigate({ search: (prev) => ({ ...prev, status: event.target.value, page: 1 }) })
              }
              className="rounded border border-[var(--border)] bg-[var(--bg)] px-2 py-1.5 text-sm text-[var(--text)] outline-none focus:border-[var(--accent)]"
            >
              <option value="">all statuses</option>
              {STATUS_OPTIONS.map((option) => (
                <option key={option} value={option}>
                  {option}
                </option>
              ))}
            </select>
          </div>
          {isPending ? (
            <p className="text-sm text-[var(--text-dim)]">loading...</p>
          ) : (
            <>
              <InvoicesTable
                invoices={data?.data ?? []}
                onView={(invoice) =>
                  navigate({ to: '/invoices/$invoiceId', params: { invoiceId: invoice.id } })
                }
                onUpdate={(invoice) =>
                  navigate({
                    to: '/invoices/$invoiceId',
                    params: { invoiceId: invoice.id },
                    search: { edit: true },
                  })
                }
                onDelete={handleDelete}
              />
              <div className="mt-4">
                <Pagination
                  page={page}
                  pageSize={PAGE_SIZE}
                  total={data?.total ?? 0}
                  onPageChange={(nextPage) =>
                    navigate({ search: (prev) => ({ ...prev, page: nextPage }) })
                  }
                />
              </div>
            </>
          )}
        </Panel>
      </div>
      {dialog}
      {newInvoiceOpen && (
        <Modal title="new invoice" onClose={() => setNewInvoiceOpen(false)}>
          <InvoiceForm
            values={newInvoiceForm}
            onChange={setNewInvoiceForm}
            onSubmit={() => createMutation.mutate(newInvoiceForm)}
            onCancel={() => setNewInvoiceOpen(false)}
            submitLabel="create"
            submitting={createMutation.isPending}
          />
        </Modal>
      )}
    </div>
  )
}
