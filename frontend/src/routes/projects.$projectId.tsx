import { useEffect, useRef, useState } from 'react'
import { Link, createFileRoute } from '@tanstack/react-router'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Panel } from '../components/Panel'
import { StatusBadge } from '../components/StatusBadge'
import { ImageSlider } from '../components/ImageSlider'
import { ProjectForm } from '../components/ProjectForm'
import type { ProjectFormValues } from '../components/ProjectForm'
import {
  PROJECT_STATUS_TONE,
  deleteProject,
  deleteProjectImage,
  fetchProject,
  updateProject,
  uploadProjectImages,
} from '../lib/api/projects'
import type { ProjectDetail } from '../lib/api/projects'
import { useConfirm } from '../hooks/use-confirm'

interface ProjectDetailSearch {
  edit: boolean
}

export const Route = createFileRoute('/projects/$projectId')({
  validateSearch: (search: Record<string, unknown>): ProjectDetailSearch => ({
    edit: search.edit === true || search.edit === 'true',
  }),
  component: ProjectDetailPage,
})

const BUTTON_CLASSES =
  'rounded border border-[var(--border)] px-3 py-1.5 text-xs text-[var(--text-dim)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)]'

function toFormValues(project: ProjectDetail): ProjectFormValues {
  return {
    clientId: project.clientId,
    name: project.name,
    description: project.description,
    category: project.category,
    url: project.url ?? '',
    status: project.status,
  }
}

function ProjectDetailPage() {
  const { projectId } = Route.useParams()
  const { edit: editFromUrl } = Route.useSearch()
  const navigate = Route.useNavigate()
  const queryClient = useQueryClient()
  const { confirm, dialog } = useConfirm()
  const fileInputRef = useRef<HTMLInputElement>(null)

  const { data: project, isLoading } = useQuery({
    queryKey: ['project', projectId],
    queryFn: () => fetchProject(projectId),
  })

  const [editing, setEditing] = useState(editFromUrl)
  const [form, setForm] = useState<ProjectFormValues | null>(null)

  useEffect(() => {
    if (project) setForm(toFormValues(project))
  }, [project])

  const updateMutation = useMutation({
    mutationFn: (values: ProjectFormValues) =>
      updateProject(projectId, {
        ...values,
        url: values.url.trim() === '' ? null : values.url.trim(),
      }),
    onSuccess: (updated) => {
      queryClient.setQueryData(['project', projectId], updated)
      queryClient.invalidateQueries({ queryKey: ['projects'] })
      queryClient.invalidateQueries({ queryKey: ['project-categories'] })
      setEditing(false)
    },
  })

  const deleteMutation = useMutation({
    mutationFn: () => deleteProject(projectId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['projects'] })
      navigate({ to: '/projects', search: { q: '', category: '' } })
    },
  })

  const uploadMutation = useMutation({
    mutationFn: (files: File[]) => uploadProjectImages(projectId, files),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['project', projectId] })
      queryClient.invalidateQueries({ queryKey: ['projects'] })
      if (fileInputRef.current) fileInputRef.current.value = ''
    },
  })

  const deleteImageMutation = useMutation({
    mutationFn: (imageId: string) => deleteProjectImage(projectId, imageId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['project', projectId] })
      queryClient.invalidateQueries({ queryKey: ['projects'] })
    },
  })

  async function handleDelete() {
    if (!project) return
    const confirmed = await confirm({
      title: 'delete project',
      message: `Delete ${project.name}? This can't be undone.`,
      confirmLabel: 'delete',
      tone: 'danger',
    })
    if (!confirmed) return
    deleteMutation.mutate()
  }

  async function handleDeleteImage(imageId: string) {
    const confirmed = await confirm({
      title: 'delete image',
      message: 'Delete this image?',
      confirmLabel: 'delete',
      tone: 'danger',
    })
    if (!confirmed) return
    deleteImageMutation.mutate(imageId)
  }

  if (isLoading) {
    return <p className="text-sm text-[var(--text-dim)]">loading...</p>
  }

  if (!project || !form) {
    return (
      <div>
        <h1 className="glow-text text-lg font-bold text-[var(--accent)]">PROJECT NOT FOUND</h1>
        <Link
          to="/projects"
          search={{ q: '', category: '' }}
          className="mt-4 inline-block text-sm text-[var(--accent)] hover:underline"
        >
          &larr; back to projects
        </Link>
      </div>
    )
  }

  return (
    <div>
      <div className="flex items-center justify-between">
        <h1 className="glow-text text-lg font-bold text-[var(--accent)]">{project.name}</h1>
        <Link
          to="/projects"
          search={{ q: '', category: '' }}
          className="text-xs text-[var(--text-dim)] hover:text-[var(--accent)]"
        >
          &larr; back to projects
        </Link>
      </div>

      <div className="mt-6 grid gap-6 lg:grid-cols-[1.4fr_1fr]">
        <div className="flex flex-col gap-4">
          <ImageSlider images={project.images} fallbackSeed={project.id} />

          <Panel title="images">
            <div className="flex flex-wrap items-center gap-2">
              <input
                ref={fileInputRef}
                type="file"
                multiple
                accept="image/jpeg,image/png,image/webp,image/gif"
                onChange={(event) => {
                  const files = Array.from(event.target.files ?? [])
                  if (files.length > 0) uploadMutation.mutate(files)
                }}
                className="text-xs text-[var(--text-dim)] file:mr-2 file:rounded file:border file:border-[var(--border)] file:bg-transparent file:px-2 file:py-1 file:text-xs file:text-[var(--text-dim)] hover:file:border-[var(--accent)] hover:file:text-[var(--accent)]"
              />
              {uploadMutation.isPending && (
                <span className="text-xs text-[var(--text-dim)]">uploading...</span>
              )}
            </div>
            {project.images.length > 0 && (
              <div className="mt-3 flex flex-wrap gap-2">
                {project.images.map((image) => (
                  <div key={image.id} className="group relative">
                    <img
                      src={image.url}
                      alt=""
                      className="h-14 w-20 rounded border border-[var(--border)] object-cover"
                    />
                    <button
                      type="button"
                      onClick={() => handleDeleteImage(image.id)}
                      aria-label="Delete image"
                      className="absolute -right-1 -top-1 rounded-full border border-[var(--border)] bg-[var(--panel)] px-1 text-[10px] text-[var(--text-dim)] opacity-0 transition-opacity hover:border-red-400 hover:text-red-400 group-hover:opacity-100"
                    >
                      &#10005;
                    </button>
                  </div>
                ))}
              </div>
            )}
          </Panel>
        </div>

        <Panel title={editing ? 'edit project' : 'project'}>
          {editing ? (
            <ProjectForm
              values={form}
              onChange={setForm}
              onSubmit={() => updateMutation.mutate(form)}
              onCancel={() => {
                setEditing(false)
                setForm(toFormValues(project))
              }}
              submitLabel="save"
              submitting={updateMutation.isPending}
            />
          ) : (
            <div className="flex flex-col gap-3 text-sm">
              <div>
                <span className="text-[var(--text-dim)]">client: </span>
                {project.clientName}
              </div>
              <div>
                <span className="text-[var(--text-dim)]">category: </span>
                {project.category}
              </div>
              <div>
                <span className="text-[var(--text-dim)]">status: </span>
                <StatusBadge label={project.status} tone={PROJECT_STATUS_TONE[project.status]} />
              </div>
              {project.description && (
                <div>
                  <span className="text-[var(--text-dim)]">description: </span>
                  {project.description}
                </div>
              )}
              {project.url && (
                <div>
                  <a
                    href={project.url}
                    target="_blank"
                    rel="noreferrer"
                    className="inline-flex items-center gap-1 text-[var(--accent)] hover:underline"
                  >
                    visit site &#8599;
                  </a>
                </div>
              )}
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
