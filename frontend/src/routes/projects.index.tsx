import { useEffect, useState } from 'react'
import { createFileRoute } from '@tanstack/react-router'
import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ProjectCard } from '../components/ProjectCard'
import { ProjectCardSkeleton } from '../components/ProjectCardSkeleton'
import { Modal } from '../components/Modal'
import { ErrorState } from '../components/ErrorState'
import { ProjectForm } from '../components/ProjectForm'
import type { ProjectFormValues } from '../components/ProjectForm'
import { createProject, fetchProjectCategories, fetchProjects } from '../lib/api/projects'
import { useDebouncedValue } from '../hooks/use-debounced-value'
import { useInfiniteScrollTrigger } from '../hooks/use-infinite-scroll-trigger'

const PAGE_SIZE = 12

const EMPTY_PROJECT: ProjectFormValues = {
  clientId: '',
  name: '',
  description: '',
  category: '',
  url: '',
  status: 'active',
}

interface ProjectsSearch {
  q: string
  category: string
}

export const Route = createFileRoute('/projects/')({
  validateSearch: (search: Record<string, unknown>): ProjectsSearch => ({
    q: typeof search.q === 'string' ? search.q : '',
    category: typeof search.category === 'string' ? search.category : '',
  }),
  component: ProjectsPage,
})

function ProjectsPage() {
  const { q, category } = Route.useSearch()
  const navigate = Route.useNavigate()
  const queryClient = useQueryClient()

  const [searchInput, setSearchInput] = useState(q)
  const debouncedSearch = useDebouncedValue(searchInput, 300)

  useEffect(() => {
    if (debouncedSearch === q) return
    navigate({ search: (prev) => ({ ...prev, q: debouncedSearch }), replace: true })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debouncedSearch])

  const { data: categories } = useQuery({
    queryKey: ['project-categories'],
    queryFn: fetchProjectCategories,
  })

  const {
    data,
    isPending,
    isError,
    refetch,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery({
    queryKey: ['projects', { search: q, category }],
    queryFn: ({ pageParam }) =>
      fetchProjects({ page: pageParam, pageSize: PAGE_SIZE, search: q, category }),
    initialPageParam: 1,
    getNextPageParam: (lastPage) =>
      lastPage.page * lastPage.pageSize < lastPage.total ? lastPage.page + 1 : undefined,
  })

  const sentinelRef = useInfiniteScrollTrigger(
    () => fetchNextPage(),
    Boolean(hasNextPage) && !isFetchingNextPage,
  )

  const projects = data?.pages.flatMap((page) => page.data) ?? []

  const [newProjectOpen, setNewProjectOpen] = useState(false)
  const [newProjectForm, setNewProjectForm] = useState<ProjectFormValues>(EMPTY_PROJECT)

  const createMutation = useMutation({
    mutationFn: (values: ProjectFormValues) =>
      createProject({ ...values, url: values.url.trim() === '' ? null : values.url.trim() }),
    onSuccess: (created) => {
      queryClient.invalidateQueries({ queryKey: ['projects'] })
      queryClient.invalidateQueries({ queryKey: ['project-categories'] })
      setNewProjectOpen(false)
      setNewProjectForm(EMPTY_PROJECT)
      navigate({ to: '/projects/$projectId', params: { projectId: created.id } })
    },
  })

  return (
    <div>
      <div className="flex items-center justify-between">
        <h1 className="glow-text text-lg font-bold text-[var(--accent)]">PROJECTS</h1>
        <button
          type="button"
          onClick={() => setNewProjectOpen(true)}
          className="rounded border border-[var(--accent)] bg-[var(--accent-dim)] px-3 py-1.5 text-xs text-[var(--accent)] transition-colors hover:bg-[var(--accent)] hover:text-[var(--bg)]"
        >
          + new
        </button>
      </div>
      <p className="mt-2 text-sm text-[var(--text-dim)]">
        // a portfolio of work across every client
      </p>

      <div className="mt-6 flex flex-col gap-3 sm:flex-row sm:items-center">
        <input
          type="text"
          value={searchInput}
          onChange={(event) => setSearchInput(event.target.value)}
          placeholder="search projects..."
          className="w-full rounded border border-[var(--border)] bg-transparent px-3 py-1.5 text-sm text-[var(--text)] outline-none placeholder:text-[var(--text-dim)] focus:border-[var(--accent)] sm:max-w-xs"
        />
        <select
          value={category}
          onChange={(event) =>
            navigate({ search: (prev) => ({ ...prev, category: event.target.value }) })
          }
          className="rounded border border-[var(--border)] bg-[var(--bg)] px-2 py-1.5 text-sm text-[var(--text)] outline-none focus:border-[var(--accent)]"
        >
          <option value="">all categories</option>
          {categories?.map((cat) => (
            <option key={cat} value={cat}>
              {cat}
            </option>
          ))}
        </select>
      </div>

      <div className="mt-6">
        {isPending ? (
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {Array.from({ length: PAGE_SIZE }).map((_, i) => (
              <ProjectCardSkeleton key={i} />
            ))}
          </div>
        ) : isError ? (
          <ErrorState
            title="COULDN'T LOAD PROJECTS"
            message="Something went wrong fetching the project list."
            onRetry={() => refetch()}
          />
        ) : projects.length === 0 ? (
          <p className="text-sm text-[var(--text-dim)]">-- no projects yet --</p>
        ) : (
          <>
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
              {projects.map((project) => (
                <ProjectCard
                  key={project.id}
                  project={project}
                  onClick={() =>
                    navigate({ to: '/projects/$projectId', params: { projectId: project.id } })
                  }
                />
              ))}
            </div>
            <div ref={sentinelRef} className="h-1" />
            {isFetchingNextPage && (
              <p className="mt-4 text-center text-sm text-[var(--text-dim)]">loading more...</p>
            )}
          </>
        )}
      </div>

      {newProjectOpen && (
        <Modal title="new project" onClose={() => setNewProjectOpen(false)}>
          <ProjectForm
            values={newProjectForm}
            onChange={setNewProjectForm}
            onSubmit={() => createMutation.mutate(newProjectForm)}
            onCancel={() => setNewProjectOpen(false)}
            submitLabel="create"
            submitting={createMutation.isPending}
          />
        </Modal>
      )}
    </div>
  )
}
