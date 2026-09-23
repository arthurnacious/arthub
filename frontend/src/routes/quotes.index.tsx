import { useEffect, useState } from 'react'
import { createFileRoute } from '@tanstack/react-router'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Panel } from '../components/Panel'
import { QuotesTable } from '../components/QuotesTable'
import { Pagination } from '../components/Pagination'
import { Modal } from '../components/Modal'
import { QuoteForm } from '../components/QuoteForm'
import { createQuote, deleteQuote, fetchQuotes } from '../lib/api/quotes'
import type { Quote, QuoteInput, QuoteStatus } from '../lib/api/quotes'
import { useDebouncedValue } from '../hooks/use-debounced-value'
import { useConfirm } from '../hooks/use-confirm'

const PAGE_SIZE = 10

const STATUS_OPTIONS: QuoteStatus[] = ['draft', 'sent', 'accepted', 'rejected', 'expired']

const EMPTY_QUOTE: QuoteInput = {
  projectId: '',
  status: 'draft',
  issueDate: null,
  expiryDate: null,
  discountType: null,
  discountValue: 0,
  notes: '',
  lineItems: [],
}

interface QuotesSearch {
  page: number
  q: string
  status: string
}

export const Route = createFileRoute('/quotes/')({
  validateSearch: (search: Record<string, unknown>): QuotesSearch => ({
    page: Number(search.page) > 0 ? Math.floor(Number(search.page)) : 1,
    q: typeof search.q === 'string' ? search.q : '',
    status: typeof search.status === 'string' ? search.status : '',
  }),
  component: QuotesPage,
})

function QuotesPage() {
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

  const { data, isLoading } = useQuery({
    queryKey: ['quotes', { page, pageSize: PAGE_SIZE, search: q, status }],
    queryFn: () => fetchQuotes({ page, pageSize: PAGE_SIZE, search: q, status }),
    placeholderData: (previousData) => previousData,
  })

  const deleteMutation = useMutation({
    mutationFn: (quote: Quote) => deleteQuote(quote.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['quotes'] })
    },
  })

  async function handleDelete(quote: Quote) {
    const confirmed = await confirm({
      title: 'delete quote',
      message: `Delete ${quote.quoteNumber}? This can't be undone.`,
      confirmLabel: 'delete',
      tone: 'danger',
    })
    if (!confirmed) return
    deleteMutation.mutate(quote)
  }

  const [newQuoteOpen, setNewQuoteOpen] = useState(false)
  const [newQuoteForm, setNewQuoteForm] = useState<QuoteInput>(EMPTY_QUOTE)

  const createMutation = useMutation({
    mutationFn: (values: QuoteInput) => createQuote(values),
    onSuccess: (created) => {
      queryClient.invalidateQueries({ queryKey: ['quotes'] })
      setNewQuoteOpen(false)
      setNewQuoteForm(EMPTY_QUOTE)
      navigate({ to: '/quotes/$quoteId', params: { quoteId: created.id } })
    },
  })

  return (
    <div>
      <div className="flex items-center justify-between">
        <h1 className="glow-text text-lg font-bold text-[var(--accent)]">QUOTES</h1>
        <button
          type="button"
          onClick={() => setNewQuoteOpen(true)}
          className="rounded border border-[var(--accent)] bg-[var(--accent-dim)] px-3 py-1.5 text-xs text-[var(--accent)] transition-colors hover:bg-[var(--accent)] hover:text-[var(--bg)]"
        >
          + new
        </button>
      </div>
      <p className="mt-2 text-sm text-[var(--text-dim)]">// estimates sent to clients for a project</p>

      <div className="mt-6">
        <Panel title="quotes">
          <div className="mb-4 flex flex-col gap-3 sm:flex-row sm:items-center">
            <input
              type="text"
              value={searchInput}
              onChange={(event) => setSearchInput(event.target.value)}
              placeholder="search quote number, project or client..."
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
          {isLoading ? (
            <p className="text-sm text-[var(--text-dim)]">loading...</p>
          ) : (
            <>
              <QuotesTable
                quotes={data?.data ?? []}
                onView={(quote) =>
                  navigate({ to: '/quotes/$quoteId', params: { quoteId: quote.id } })
                }
                onUpdate={(quote) =>
                  navigate({
                    to: '/quotes/$quoteId',
                    params: { quoteId: quote.id },
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
      {newQuoteOpen && (
        <Modal title="new quote" onClose={() => setNewQuoteOpen(false)}>
          <QuoteForm
            values={newQuoteForm}
            onChange={setNewQuoteForm}
            onSubmit={() => createMutation.mutate(newQuoteForm)}
            onCancel={() => setNewQuoteOpen(false)}
            submitLabel="create"
            submitting={createMutation.isPending}
          />
        </Modal>
      )}
    </div>
  )
}
