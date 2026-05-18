import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Project {
  id: number
  name: string
  createdAt: number
  status: string
  targetVenue?: string
  docPath?: string
  folderPath: string
}

export interface SectionAnnotationEntry {
  annotationId: number
  note?: string
}

export interface SectionNoteEntry {
  noteId: number
}

export interface ProjectSection {
  id: number
  projectId: number
  title: string
  position: number
  note?: string
  annotations: SectionAnnotationEntry[]
  noteEntries: SectionNoteEntry[]
}

export interface ProjectNote {
  id: number
  projectId: number
  body: string
  code?: string
  language?: string
  createdAt: number
}

// ── Module-level singletons ───────────────────────────────────────────────────

const projects  = ref<Project[]>([])
const currentId = ref<number | null>(null)
const sections  = ref<ProjectSection[]>([])
const sourceIds = ref<number[]>([])
const notes     = ref<ProjectNote[]>([])
const loading   = ref(false)

export function useWorkbench() {

  const currentProject = computed(() =>
    projects.value.find(p => p.id === currentId.value) ?? null
  )

  // ── Projects ────────────────────────────────────────────────────────────────

  async function loadProjects(): Promise<void> {
    loading.value = true
    try {
      projects.value = await invoke<Project[]>('get_projects')
      if (projects.value.length > 0 && currentId.value === null) {
        await selectProject(projects.value[0].id)
      } else if (currentId.value !== null) {
        await loadProjectData(currentId.value)
      }
    } finally {
      loading.value = false
    }
  }

  async function createProject(name: string): Promise<Project> {
    const p = await invoke<Project>('create_project', { name })
    projects.value.unshift(p)
    await selectProject(p.id)
    return p
  }

  async function deleteProject(id: number): Promise<void> {
    await invoke('delete_project', { id })
    projects.value = projects.value.filter(p => p.id !== id)
    if (currentId.value === id) {
      if (projects.value.length > 0) {
        await selectProject(projects.value[0].id)
      } else {
        currentId.value = null
        sections.value = []
        sourceIds.value = []
        notes.value = []
      }
    }
  }

  async function renameProject(id: number, name: string): Promise<void> {
    await invoke('rename_project', { id, name })
    const idx = projects.value.findIndex(p => p.id === id)
    if (idx !== -1) projects.value[idx] = { ...projects.value[idx], name }
  }

  async function updateProjectMeta(id: number, status: string, targetVenue?: string): Promise<void> {
    await invoke('update_project_meta', { id, status, targetVenue: targetVenue ?? null })
    const idx = projects.value.findIndex(p => p.id === id)
    if (idx !== -1) projects.value[idx] = { ...projects.value[idx], status, targetVenue }
  }

  async function setProjectDocPath(id: number, docPath: string | null): Promise<void> {
    await invoke('set_project_doc_path', { id, docPath })
    const idx = projects.value.findIndex(p => p.id === id)
    if (idx !== -1) projects.value[idx] = { ...projects.value[idx], docPath: docPath ?? undefined }
  }

  async function selectProject(id: number): Promise<void> {
    currentId.value = id
    await loadProjectData(id)
  }

  async function loadProjectData(id: number): Promise<void> {
    const [sids, secs, nts] = await Promise.all([
      invoke<number[]>('get_project_source_ids', { projectId: id }),
      invoke<ProjectSection[]>('get_project_sections', { projectId: id }),
      invoke<ProjectNote[]>('get_project_notes', { projectId: id }),
    ])
    sourceIds.value = sids
    sections.value  = secs
    notes.value     = nts
  }

  // ── Sources ─────────────────────────────────────────────────────────────────

  async function addSource(itemId: number): Promise<void> {
    if (!currentId.value || sourceIds.value.includes(itemId)) return
    await invoke('add_project_source', { projectId: currentId.value, itemId })
    sourceIds.value = [...sourceIds.value, itemId]
  }

  async function removeSource(itemId: number): Promise<void> {
    if (!currentId.value) return
    await invoke('remove_project_source', { projectId: currentId.value, itemId })
    sourceIds.value = sourceIds.value.filter(id => id !== itemId)
  }

  // ── Sections ────────────────────────────────────────────────────────────────

  async function addSection(title: string): Promise<void> {
    if (!currentId.value) return
    const position = sections.value.length
    const sec = await invoke<ProjectSection>('create_project_section', {
      projectId: currentId.value, title, position,
    })
    sections.value = [...sections.value, sec]
  }

  async function renameSection(id: number, title: string): Promise<void> {
    const sec = sections.value.find(s => s.id === id)
    if (!sec) return
    await invoke('update_project_section', { id, title, note: sec.note ?? null })
    const idx = sections.value.findIndex(s => s.id === id)
    if (idx !== -1) sections.value[idx] = { ...sections.value[idx], title }
  }

  async function deleteSection(id: number): Promise<void> {
    await invoke('delete_project_section', { id })
    sections.value = sections.value.filter(s => s.id !== id)
  }

  async function moveSectionUp(id: number): Promise<void> {
    const idx = sections.value.findIndex(s => s.id === id)
    if (idx <= 0) return
    const arr = [...sections.value]
    ;[arr[idx - 1], arr[idx]] = [arr[idx], arr[idx - 1]]
    sections.value = arr
    await invoke('reorder_project_sections', {
      projectId: currentId.value,
      orderedIds: arr.map(s => s.id),
    })
  }

  async function moveSectionDown(id: number): Promise<void> {
    const idx = sections.value.findIndex(s => s.id === id)
    if (idx === -1 || idx >= sections.value.length - 1) return
    const arr = [...sections.value]
    ;[arr[idx], arr[idx + 1]] = [arr[idx + 1], arr[idx]]
    sections.value = arr
    await invoke('reorder_project_sections', {
      projectId: currentId.value,
      orderedIds: arr.map(s => s.id),
    })
  }

  // ── Section annotations ──────────────────────────────────────────────────────

  async function dropAnnotation(sectionId: number, annotationId: number): Promise<void> {
    await invoke('add_annotation_to_section', { sectionId, annotationId })
    const idx = sections.value.findIndex(s => s.id === sectionId)
    if (idx !== -1 && !sections.value[idx].annotations.some(e => e.annotationId === annotationId)) {
      sections.value[idx] = {
        ...sections.value[idx],
        annotations: [...sections.value[idx].annotations, { annotationId }],
      }
    }
  }

  async function liftAnnotation(sectionId: number, annotationId: number): Promise<void> {
    await invoke('remove_annotation_from_section', { sectionId, annotationId })
    const idx = sections.value.findIndex(s => s.id === sectionId)
    if (idx !== -1) {
      sections.value[idx] = {
        ...sections.value[idx],
        annotations: sections.value[idx].annotations.filter(e => e.annotationId !== annotationId),
      }
    }
  }

  async function updateSectionAnnotationNote(
    sectionId: number,
    annotationId: number,
    note: string,
  ): Promise<void> {
    await invoke('update_section_annotation_note', {
      sectionId,
      annotationId,
      note: note || null,
    })
    const secIdx = sections.value.findIndex(s => s.id === sectionId)
    if (secIdx === -1) return
    const annIdx = sections.value[secIdx].annotations.findIndex(e => e.annotationId === annotationId)
    if (annIdx === -1) return
    const updated = [...sections.value[secIdx].annotations]
    updated[annIdx] = { ...updated[annIdx], note: note || undefined }
    sections.value[secIdx] = { ...sections.value[secIdx], annotations: updated }
  }

  // ── Project notes ────────────────────────────────────────────────────────────

  async function createNote(body: string, code?: string, language?: string): Promise<ProjectNote> {
    if (!currentId.value) throw new Error('No active project')
    const n = await invoke<ProjectNote>('create_project_note', {
      projectId: currentId.value,
      body,
      code: code ?? null,
      language: language ?? null,
    })
    notes.value = [n, ...notes.value]
    return n
  }

  async function updateNote(id: number, body: string, code?: string, language?: string): Promise<void> {
    await invoke('update_project_note', { id, body, code: code ?? null, language: language ?? null })
    const idx = notes.value.findIndex(n => n.id === id)
    if (idx !== -1) {
      notes.value[idx] = { ...notes.value[idx], body, code, language }
    }
  }

  async function deleteNote(id: number): Promise<void> {
    await invoke('delete_project_note', { id })
    notes.value = notes.value.filter(n => n.id !== id)
    // Remove from all sections (cascade handles DB; update local state)
    sections.value = sections.value.map(s => ({
      ...s,
      noteEntries: s.noteEntries.filter(e => e.noteId !== id),
    }))
  }

  async function dropNote(sectionId: number, noteId: number): Promise<void> {
    await invoke('add_note_to_section', { sectionId, noteId })
    const idx = sections.value.findIndex(s => s.id === sectionId)
    if (idx !== -1 && !sections.value[idx].noteEntries.some(e => e.noteId === noteId)) {
      sections.value[idx] = {
        ...sections.value[idx],
        noteEntries: [...sections.value[idx].noteEntries, { noteId }],
      }
    }
  }

  async function liftNote(sectionId: number, noteId: number): Promise<void> {
    await invoke('remove_note_from_section', { sectionId, noteId })
    const idx = sections.value.findIndex(s => s.id === sectionId)
    if (idx !== -1) {
      sections.value[idx] = {
        ...sections.value[idx],
        noteEntries: sections.value[idx].noteEntries.filter(e => e.noteId !== noteId),
      }
    }
  }

  return {
    projects, currentId, currentProject, sections, sourceIds, notes, loading,
    loadProjects, createProject, deleteProject, renameProject, selectProject,
    updateProjectMeta, setProjectDocPath,
    addSource, removeSource,
    addSection, renameSection, deleteSection,
    moveSectionUp, moveSectionDown,
    dropAnnotation, liftAnnotation, updateSectionAnnotationNote,
    createNote, updateNote, deleteNote, dropNote, liftNote,
  }
}
