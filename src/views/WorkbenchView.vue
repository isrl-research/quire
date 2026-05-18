<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useWorkbench, type ProjectSection, type ProjectNote } from '../composables/useWorkbench'
import { useAllAnnotations, type AnnotationWithSource } from '../composables/useAnnotations'
import { emitter } from '../events'

const router = useRouter()

const wb  = useWorkbench()
const ann = useAllAnnotations()

onMounted(async () => {
  await Promise.all([wb.loadProjects(), ann.loadAll()])
})

// ── Source items (for picker display) ─────────────────────────────────────────

interface ItemMeta { id: number; title?: string; authors?: string; year?: string; key: string }
const allItems = ref<ItemMeta[]>([])

onMounted(async () => {
  try {
    allItems.value = await invoke<ItemMeta[]>('get_library_items')
  } catch { /* no items yet */ }
})

const sourceItems = computed(() =>
  allItems.value.filter(i => wb.sourceIds.value.includes(i.id))
)

// ── Annotations filtered to this project's sources ────────────────────────────

const sourceAnnotations = computed(() =>
  ann.allAnnotations.value.filter(a => wb.sourceIds.value.includes(a.itemId))
)

// Annotations already dropped anywhere in this project
const droppedIds = computed(() => {
  const set = new Set<number>()
  for (const s of wb.sections.value) s.annotations.forEach(e => set.add(e.annotationId))
  return set
})

// ── Source picker ─────────────────────────────────────────────────────────────

const pickerOpen  = ref(false)
const pickerQuery = ref('')

const pickerResults = computed(() => {
  const q = pickerQuery.value.trim().toLowerCase()
  return allItems.value
    .filter(i => !wb.sourceIds.value.includes(i.id))
    .filter(i =>
      !q ||
      (i.title?.toLowerCase().includes(q)) ||
      (i.authors?.toLowerCase().includes(q)) ||
      i.key.toLowerCase().includes(q)
    )
    .slice(0, 12)
})

async function pickSource(id: number) {
  await wb.addSource(id)
  pickerQuery.value = ''
}

function closePicker() {
  pickerOpen.value = false
  pickerQuery.value = ''
}

// ── Export .bib ───────────────────────────────────────────────────────────────

const exportStatus = ref<'idle' | 'ok' | 'err'>('idle')
let exportTimer: ReturnType<typeof setTimeout> | null = null

async function exportBib() {
  const ids = wb.sourceIds.value
  if (!ids.length) return
  try {
    const saved = await invoke<boolean>('export_project_bib_dialog', {
      itemIds: ids,
      projectName: wb.currentProject.value?.name ?? 'project',
    })
    exportStatus.value = saved ? 'ok' : 'idle'
  } catch {
    exportStatus.value = 'err'
  }
  if (exportTimer) clearTimeout(exportTimer)
  exportTimer = setTimeout(() => { exportStatus.value = 'idle' }, 2000)
}

// ── Copy for AI ───────────────────────────────────────────────────────────────

const copyStatus = ref<'idle' | 'ok'>('idle')
let copyTimer: ReturnType<typeof setTimeout> | null = null

async function copyForAi() {
  const project = wb.currentProject.value
  if (!project) return

  // Get .bib text for the project's sources
  const bibText = wb.sourceIds.value.length
    ? await invoke<string>('get_bib_text_for_items', { itemIds: wb.sourceIds.value })
    : ''

  const lines: string[] = []
  lines.push('You are an academic writing assistant. Using the research outline and annotated sources below, write a complete academic paper draft.')
  lines.push('Use [@citeKey] citation syntax wherever you reference a source. Return Markdown only — no preamble, no commentary.\n')
  lines.push(`# Project: ${project.name}\n`)
  lines.push('## Outline\n')

  for (const sec of wb.sections.value) {
    lines.push(`### ${sec.title}`)

    if (sec.annotations.length) {
      lines.push('\n**Annotations:**')
      for (const entry of sec.annotations) {
        const a = ann.allAnnotations.value.find(x => x.id === entry.annotationId)
        if (!a) continue
        const text = a.selectedText?.trim() || a.noteText?.trim() || ''
        const cite = a.itemKey ? `[@${a.itemKey}]` : ''
        const page = a.page ? ` p. ${a.page}` : ''
        lines.push(`- "${text}"${cite ? ' ' + cite : ''}${page}`)
        if (entry.note?.trim()) lines.push(`  → ${entry.note.trim()}`)
      }
    }

    if (sec.noteEntries.length) {
      lines.push('\n**Research notes:**')
      for (const ne of sec.noteEntries) {
        const n = wb.notes.value.find(x => x.id === ne.noteId)
        if (!n) continue
        lines.push(`- ${n.body}`)
        if (n.code) lines.push(`  \`\`\`${n.language || ''}\n  ${n.code}\n  \`\`\``)
      }
    }

    lines.push('')
  }

  // Project-level notes not assigned to any section
  const unassignedNotes = wb.notes.value.filter(n => {
    const used = new Set(wb.sections.value.flatMap(s => s.noteEntries.map(e => e.noteId)))
    return !used.has(n.id)
  })
  if (unassignedNotes.length) {
    lines.push('## General notes\n')
    for (const n of unassignedNotes) lines.push(`- ${n.body}`)
    lines.push('')
  }

  if (bibText.trim()) {
    lines.push('---\n')
    lines.push('## Bibliography\n')
    lines.push(bibText)
  }

  await navigator.clipboard.writeText(lines.join('\n'))
  copyStatus.value = 'ok'
  if (copyTimer) clearTimeout(copyTimer)
  copyTimer = setTimeout(() => { copyStatus.value = 'idle' }, 2000)
}

// ── Write from sections ───────────────────────────────────────────────────────

function writeFromSections() {
  if (wb.sections.value.length) {
    const html = wb.sections.value.map(s => `<h2>${s.title}</h2><p></p>`).join('')
    emitter.emit('doc:opened', { path: '', content: html })
  }
  router.push('/write')
}

// ── Inline notes (below outline) ─────────────────────────────────────────────

const quickNoteOpen = ref(false)
const quickNoteBody = ref('')
const quickNoteInput = ref<HTMLTextAreaElement | null>(null)

async function submitQuickNote() {
  const body = quickNoteBody.value.trim()
  quickNoteOpen.value = false
  quickNoteBody.value = ''
  if (body) await wb.createNote(body)
}

function openQuickNote() {
  quickNoteOpen.value = true
  quickNoteBody.value = ''
  nextTick(() => quickNoteInput.value?.focus())
}

// ── Section title editing ─────────────────────────────────────────────────────

const editingSectionId = ref<number | null>(null)
const sectionDraft     = ref('')

function startEditSection(sec: ProjectSection) {
  editingSectionId.value = sec.id
  sectionDraft.value = sec.title
  nextTick(() => (document.querySelector('.section-title-input') as HTMLInputElement | null)?.select())
}

async function commitEditSection(id: number) {
  await wb.renameSection(id, sectionDraft.value.trim() || 'Untitled section')
  editingSectionId.value = null
}

// ── Add section ───────────────────────────────────────────────────────────────

const addingSectionTitle = ref('')
const addSectionOpen     = ref(false)
const addSectionInput    = ref<HTMLInputElement | null>(null)

function openAddSection() {
  addSectionOpen.value  = true
  addingSectionTitle.value = ''
  nextTick(() => addSectionInput.value?.focus())
}

async function submitAddSection() {
  const t = addingSectionTitle.value.trim()
  if (t) await wb.addSection(t)
  addSectionOpen.value = false
  addingSectionTitle.value = ''
}

// ── {{ trigger: annotation + note picker per section ─────────────────────────

type Suggestion =
  | { kind: 'annotation'; data: AnnotationWithSource }
  | { kind: 'note';       data: ProjectNote }

const sectionQuery    = ref<Record<number, string>>({})
const activeSectionId = ref<number | null>(null)
const focusedIdx      = ref(0)

const droppedNoteIds = computed(() => {
  const set = new Set<number>()
  for (const s of wb.sections.value) s.noteEntries.forEach(e => set.add(e.noteId))
  return set
})

const suggestions = computed((): Suggestion[] => {
  if (activeSectionId.value === null) return []
  const raw = sectionQuery.value[activeSectionId.value] ?? ''
  const trigger = raw.lastIndexOf('{{')
  if (trigger === -1) return []
  const search = raw.slice(trigger + 2).toLowerCase().trim()

  const annSugs: Suggestion[] = sourceAnnotations.value
    .filter(a => !droppedIds.value.has(a.id))
    .filter(a =>
      !search ||
      (a.selectedText?.toLowerCase().includes(search)) ||
      (a.noteText?.toLowerCase().includes(search)) ||
      (a.itemTitle?.toLowerCase().includes(search)) ||
      (a.itemAuthors?.toLowerCase().includes(search))
    )
    .slice(0, 6)
    .map(a => ({ kind: 'annotation', data: a }))

  const noteSugs: Suggestion[] = wb.notes.value
    .filter(n => !droppedNoteIds.value.has(n.id))
    .filter(n =>
      !search ||
      n.body.toLowerCase().includes(search) ||
      (n.code?.toLowerCase().includes(search))
    )
    .slice(0, 4)
    .map(n => ({ kind: 'note', data: n }))

  return [...annSugs, ...noteSugs]
})

function onQueryInput(e: Event, sectionId: number) {
  const val = (e.target as HTMLInputElement).value
  sectionQuery.value = { ...sectionQuery.value, [sectionId]: val }
  activeSectionId.value = sectionId
  focusedIdx.value = 0
}

function onQueryFocus(sectionId: number) {
  activeSectionId.value = sectionId
}

function onQueryBlur() {
  setTimeout(() => { activeSectionId.value = null }, 180)
}

function clearQuery(sectionId: number) {
  sectionQuery.value = { ...sectionQuery.value, [sectionId]: '' }
  activeSectionId.value = null
}

function suggestDown() { focusedIdx.value = Math.min(focusedIdx.value + 1, suggestions.value.length - 1) }
function suggestUp()   { focusedIdx.value = Math.max(focusedIdx.value - 1, 0) }

async function selectSuggestion(sectionId: number, sug: Suggestion) {
  if (sug.kind === 'annotation') {
    await wb.dropAnnotation(sectionId, sug.data.id)
  } else {
    await wb.dropNote(sectionId, sug.data.id)
  }
  clearQuery(sectionId)
}

async function selectFocused(sectionId: number) {
  const s = suggestions.value[focusedIdx.value]
  if (s) await selectSuggestion(sectionId, s)
}

// ── Sidebar annotation groups ─────────────────────────────────────────────────

interface AnnGroup { itemId: number; label: string; sub: string; anns: AnnotationWithSource[] }

const sidebarGroups = computed<AnnGroup[]>(() => {
  const map = new Map<number, AnnGroup>()
  for (const a of sourceAnnotations.value) {
    if (!map.has(a.itemId)) {
      map.set(a.itemId, {
        itemId: a.itemId,
        label: a.itemTitle ?? a.itemKey,
        sub: shortAuthors(a.itemAuthors) + (a.itemYear ? ' ' + a.itemYear : ''),
        anns: [],
      })
    }
    map.get(a.itemId)!.anns.push(a)
  }
  return [...map.values()]
})

// ── Sidebar tabs & search ─────────────────────────────────────────────────────

const sidebarSearch    = ref('')
const sidebarCollapsed = ref(false)
const sidebarTab       = ref<'literature' | 'notes'>('literature')

// ── My Notes ─────────────────────────────────────────────────────────────────

const newNoteBody     = ref('')
const newNoteCode     = ref('')
const newNoteLang     = ref('')
const newNoteOpen     = ref(false)
const editingNoteId   = ref<number | null>(null)
const editNoteBody    = ref('')
const editNoteCode    = ref('')
const editNoteLang    = ref('')
const expandedCodeIds = ref(new Set<number>())

async function submitNewNote() {
  const body = newNoteBody.value.trim()
  if (!body) return
  const code = newNoteCode.value.trim() || undefined
  const lang = newNoteLang.value.trim() || undefined
  await wb.createNote(body, code, lang)
  newNoteBody.value = ''
  newNoteCode.value = ''
  newNoteLang.value = ''
  newNoteOpen.value = false
}

function startEditNote(note: ProjectNote) {
  editingNoteId.value = note.id
  editNoteBody.value  = note.body
  editNoteCode.value  = note.code ?? ''
  editNoteLang.value  = note.language ?? ''
}

async function commitEditNote() {
  if (editingNoteId.value === null) return
  const body = editNoteBody.value.trim()
  const code = editNoteCode.value.trim() || undefined
  const lang = editNoteLang.value.trim() || undefined
  await wb.updateNote(editingNoteId.value, body || '—', code, lang)
  editingNoteId.value = null
}

function toggleCodeExpand(id: number) {
  if (expandedCodeIds.value.has(id)) {
    expandedCodeIds.value.delete(id)
  } else {
    expandedCodeIds.value.add(id)
  }
  expandedCodeIds.value = new Set(expandedCodeIds.value)
}

const filteredGroups = computed<AnnGroup[]>(() => {
  const q = sidebarSearch.value.trim().toLowerCase()
  if (!q) return sidebarGroups.value
  return sidebarGroups.value
    .map(g => ({ ...g, anns: g.anns.filter(a =>
      (a.selectedText?.toLowerCase().includes(q)) ||
      (a.noteText?.toLowerCase().includes(q))
    )}))
    .filter(g => g.anns.length > 0)
})

// ── Helpers ───────────────────────────────────────────────────────────────────

function shortAuthors(authors?: string): string {
  if (!authors) return ''
  const parts = authors.split(/[,;&]|and\s/i).map(s => s.trim()).filter(Boolean)
  if (!parts.length) return ''
  const last = parts[0].split(/\s+/).pop() ?? parts[0]
  return parts.length > 2 ? `${last} et al.` : parts.length === 2
    ? `${last} & ${parts[1].split(/\s+/).pop() ?? parts[1]}`
    : last
}

function annById(id: number): AnnotationWithSource | undefined {
  return ann.allAnnotations.value.find(a => a.id === id)
}

function annText(a?: AnnotationWithSource): string {
  if (!a) return ''
  return a.selectedText?.trim() || a.noteText?.trim() || '—'
}

function noteById(id: number): ProjectNote | undefined {
  return wb.notes.value.find(n => n.id === id)
}

function relativeTime(unixSec: number): string {
  const diff = Math.floor(Date.now() / 1000) - unixSec
  if (diff < 60)    return 'just now'
  if (diff < 3600)  return `${Math.floor(diff / 60)}m ago`
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`
  if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`
  return new Date(unixSec * 1000).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
}

function autoResize(e: Event) {
  const el = e.target as HTMLTextAreaElement
  el.style.height = 'auto'
  el.style.height = el.scrollHeight + 'px'
}
</script>

<template>
  <div class="wb" @click.self="closePicker">

    <!-- ── Empty: no project selected ──────────────────────────────────────── -->
    <div v-if="!wb.currentProject.value && !wb.loading.value" class="wb-empty">
      <div class="wb-empty-inner">
        <svg width="32" height="32" viewBox="0 0 32 32" fill="none" stroke="currentColor"
             stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" opacity="0.35">
          <rect x="4" y="6" width="24" height="20" rx="2"/>
          <line x1="10" y1="13" x2="22" y2="13"/>
          <line x1="10" y1="18" x2="17" y2="18"/>
        </svg>
        <p class="wb-empty-title">No project open</p>
        <p class="wb-empty-sub">Create or select a project to start your outline.</p>
        <button class="btn-primary" @click="router.push('/projects')">← Go to Projects</button>
      </div>
    </div>

    <!-- ── Main workbench ────────────────────────────────────────────────────── -->
    <template v-else-if="wb.currentProject.value">

      <!-- Header bar -->
      <div class="wb-header">
        <div class="wb-header-left">
          <!-- Project name -->
          <span class="project-name-label">{{ wb.currentProject.value?.name }}</span>

          <div class="header-sep"></div>

          <!-- Sources -->
          <div class="sources-row">
            <span class="sources-label">Sources</span>
            <div class="source-chips">
              <div v-for="item in sourceItems" :key="item.id" class="source-chip">
                <span class="chip-text">{{ item.authors ? shortAuthors(item.authors) : item.key }}{{ item.year ? ' ' + item.year : '' }}</span>
                <button class="chip-remove" @click="wb.removeSource(item.id)" title="Remove source">
                  <svg width="8" height="8" viewBox="0 0 8 8" fill="none" stroke="currentColor"
                       stroke-width="1.5" stroke-linecap="round">
                    <line x1="1" y1="1" x2="7" y2="7"/><line x1="7" y1="1" x2="1" y2="7"/>
                  </svg>
                </button>
              </div>
              <div class="source-picker-wrap">
                <button class="chip-add" @click="pickerOpen = !pickerOpen" title="Add source">
                  <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor"
                       stroke-width="1.5" stroke-linecap="round">
                    <line x1="5.5" y1="1" x2="5.5" y2="10"/>
                    <line x1="1" y1="5.5" x2="10" y2="5.5"/>
                  </svg>
                </button>
                <div v-if="pickerOpen" class="source-picker">
                  <input
                    v-model="pickerQuery"
                    class="picker-search"
                    placeholder="Search library…"
                    autofocus
                    @keydown.escape="closePicker"
                  />
                  <div class="picker-list">
                    <div v-if="pickerResults.length === 0" class="picker-empty">No matches</div>
                    <button
                      v-for="item in pickerResults"
                      :key="item.id"
                      class="picker-row"
                      @click="pickSource(item.id)"
                    >
                      <span class="picker-title">{{ item.title ?? item.key }}</span>
                      <span class="picker-sub">{{ shortAuthors(item.authors) }}{{ item.year ? ' · ' + item.year : '' }}</span>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="wb-header-right">
          <button
            class="btn-ghost btn-sm"
            :disabled="wb.sourceIds.value.length === 0"
            :title="wb.sourceIds.value.length === 0 ? 'Add sources first' : 'Export sources as .bib file'"
            @click="exportBib"
          >
            {{ exportStatus === 'ok' ? 'Saved ✓' : exportStatus === 'err' ? 'Error' : 'Export .bib' }}
          </button>
          <button
            class="btn-ghost btn-sm"
            :disabled="wb.sections.value.length === 0"
            :title="wb.sections.value.length === 0 ? 'Add sections first' : 'Copy workbench as AI prompt'"
            @click="copyForAi"
          >
            {{ copyStatus === 'ok' ? 'Copied ✓' : 'Copy for AI' }}
          </button>
          <button class="btn-primary btn-sm" @click="writeFromSections">
            → Write
          </button>
        </div>
      </div>

      <!-- Body: sections + sidebar -->
      <div class="wb-body">

        <!-- ── Left: section cards ────────────────────────────────────────── -->
        <div class="wb-main">
          <div class="sections-list">

            <div
              v-for="(sec, idx) in wb.sections.value"
              :key="sec.id"
              class="section-card"
            >
              <!-- Section header -->
              <div class="section-head">
                <div class="section-num">{{ idx + 1 }}</div>
                <template v-if="editingSectionId === sec.id">
                  <input
                    v-model="sectionDraft"
                    class="section-title-input"
                    @keydown.enter="commitEditSection(sec.id)"
                    @keydown.escape="editingSectionId = null"
                    @blur="commitEditSection(sec.id)"
                  />
                </template>
                <span v-else class="section-title" @dblclick="startEditSection(sec)">{{ sec.title }}</span>
                <div class="section-actions">
                  <button class="icon-btn" title="Move up"
                          :disabled="idx === 0"
                          @click="wb.moveSectionUp(sec.id)">
                    <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor"
                         stroke-width="1.5" stroke-linecap="round">
                      <path d="M2 7l3.5-3.5L9 7"/>
                    </svg>
                  </button>
                  <button class="icon-btn" title="Move down"
                          :disabled="idx === wb.sections.value.length - 1"
                          @click="wb.moveSectionDown(sec.id)">
                    <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor"
                         stroke-width="1.5" stroke-linecap="round">
                      <path d="M2 4l3.5 3.5L9 4"/>
                    </svg>
                  </button>
                  <button class="icon-btn icon-btn--del" title="Delete section"
                          @click="wb.deleteSection(sec.id)">
                    <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor"
                         stroke-width="1.5" stroke-linecap="round">
                      <line x1="1" y1="1" x2="10" y2="10"/><line x1="10" y1="1" x2="1" y2="10"/>
                    </svg>
                  </button>
                </div>
              </div>

              <!-- Dropped items + {{ trigger -->
              <div class="section-body">

                <!-- Dropped annotations -->
                <div
                  v-for="entry in sec.annotations"
                  :key="'ann-' + entry.annotationId"
                  class="dropped-ann"
                >
                  <div class="dropped-stripe" :style="{ background: annById(entry.annotationId)?.color }"></div>
                  <div class="dropped-body">
                    <div class="dropped-text">{{ annText(annById(entry.annotationId)) }}</div>
                    <div class="dropped-meta">
                      {{ shortAuthors(annById(entry.annotationId)?.itemAuthors) }}{{ annById(entry.annotationId)?.itemYear ? ' ' + annById(entry.annotationId)?.itemYear : '' }}
                      · p.&nbsp;{{ annById(entry.annotationId)?.page }}
                    </div>
                    <textarea
                      class="dropped-note"
                      :value="entry.note ?? ''"
                      placeholder="Add a note…"
                      rows="1"
                      @input="autoResize($event)"
                      @change="wb.updateSectionAnnotationNote(sec.id, entry.annotationId, ($event.target as HTMLTextAreaElement).value)"
                    ></textarea>
                  </div>
                  <button class="dropped-remove" title="Remove"
                          @click="wb.liftAnnotation(sec.id, entry.annotationId)">
                    <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor"
                         stroke-width="1.5" stroke-linecap="round">
                      <line x1="1" y1="1" x2="8" y2="8"/><line x1="8" y1="1" x2="1" y2="8"/>
                    </svg>
                  </button>
                </div>

                <!-- Dropped notes -->
                <div
                  v-for="ne in sec.noteEntries"
                  :key="'note-' + ne.noteId"
                  class="dropped-note-block"
                >
                  <template v-if="noteById(ne.noteId)">
                    <div class="dropped-note-stripe"></div>
                    <div class="dropped-body">
                      <div class="dropped-note-ts">{{ relativeTime(noteById(ne.noteId)!.createdAt) }}</div>
                      <div class="dropped-text">{{ noteById(ne.noteId)!.body }}</div>
                      <template v-if="noteById(ne.noteId)!.code">
                        <button class="code-toggle" @click="toggleCodeExpand(ne.noteId)">
                          <span class="code-lang">{{ noteById(ne.noteId)!.language || 'code' }}</span>
                          <svg width="8" height="8" viewBox="0 0 8 8" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                            <path v-if="expandedCodeIds.has(ne.noteId)" d="M1 5l3-3 3 3"/>
                            <path v-else d="M1 3l3 3 3-3"/>
                          </svg>
                        </button>
                        <pre v-if="expandedCodeIds.has(ne.noteId)" class="code-block">{{ noteById(ne.noteId)!.code }}</pre>
                      </template>
                    </div>
                    <button class="dropped-remove" title="Remove"
                            @click="wb.liftNote(sec.id, ne.noteId)">
                      <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor"
                           stroke-width="1.5" stroke-linecap="round">
                        <line x1="1" y1="1" x2="8" y2="8"/><line x1="8" y1="1" x2="1" y2="8"/>
                      </svg>
                    </button>
                  </template>
                </div>

                <!-- {{ annotation/note trigger -->
                <div class="section-trigger-wrap">
                  <input
                    :value="sectionQuery[sec.id] ?? ''"
                    class="section-trigger-input"
                    :placeholder="(sec.annotations.length + sec.noteEntries.length) === 0 ? 'Type {{ to add an annotation or note…' : '{{'"
                    @input="onQueryInput($event, sec.id)"
                    @focus="onQueryFocus(sec.id)"
                    @blur="onQueryBlur"
                    @keydown.escape.prevent="clearQuery(sec.id)"
                    @keydown.down.prevent="suggestDown"
                    @keydown.up.prevent="suggestUp"
                    @keydown.enter.prevent="selectFocused(sec.id)"
                  />
                  <div
                    v-if="activeSectionId === sec.id && suggestions.length > 0"
                    class="sug-list"
                  >
                    <div
                      v-for="(sug, i) in suggestions"
                      :key="sug.kind + '-' + sug.data.id"
                      class="sug-row"
                      :class="{ 'sug-focused': i === focusedIdx }"
                      @mousedown.prevent="selectSuggestion(sec.id, sug)"
                    >
                      <template v-if="sug.kind === 'annotation'">
                        <div class="sug-stripe" :style="{ background: sug.data.color }"></div>
                        <div class="sug-body">
                          <div class="sug-text">{{ annText(sug.data) }}</div>
                          <div class="sug-meta">{{ shortAuthors(sug.data.itemAuthors) }}{{ sug.data.itemYear ? ' ' + sug.data.itemYear : '' }} · p.{{ sug.data.page }}</div>
                        </div>
                      </template>
                      <template v-else>
                        <div class="sug-stripe sug-stripe--note"></div>
                        <div class="sug-body">
                          <div class="sug-text">{{ sug.data.body }}</div>
                          <div class="sug-meta">My note · {{ relativeTime(sug.data.createdAt) }}{{ sug.data.code ? ' · has code' : '' }}</div>
                        </div>
                      </template>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Add section -->
            <div v-if="addSectionOpen" class="add-section-row">
              <input
                ref="addSectionInput"
                v-model="addingSectionTitle"
                class="add-section-input"
                placeholder="Section title…"
                @keydown.enter="submitAddSection"
                @keydown.escape="addSectionOpen = false"
                @blur="submitAddSection"
              />
            </div>
            <button v-else class="add-section-btn" @click="openAddSection">
              <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor"
                   stroke-width="1.5" stroke-linecap="round">
                <line x1="5.5" y1="1" x2="5.5" y2="10"/>
                <line x1="1" y1="5.5" x2="10" y2="5.5"/>
              </svg>
              Add section
            </button>

          </div>

          <!-- ── Notes panel ──────────────────────────────────────────────── -->
          <div class="notes-panel">
            <div class="notes-panel-head">
              <span class="notes-panel-label">Notes</span>
              <span class="notes-panel-count">{{ wb.notes.value.length }}</span>
            </div>

            <!-- Existing notes -->
            <div class="notes-panel-list">
              <div
                v-for="note in wb.notes.value"
                :key="note.id"
                class="note-inline-card"
                :class="{ 'is-dropped': droppedNoteIds.has(note.id) }"
              >
                <div class="note-inline-ts">{{ relativeTime(note.createdAt) }}</div>
                <div class="note-inline-body">{{ note.body }}</div>
                <span v-if="note.code" class="note-inline-code-badge">{{ note.language || 'code' }}</span>
                <div class="note-inline-actions">
                  <button class="icon-btn" title="Edit" @click="startEditNote(note)">
                    <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor"
                         stroke-width="1.5" stroke-linecap="round">
                      <path d="M7 1L9 3L3.5 8.5H1.5V6.5L7 1Z"/>
                    </svg>
                  </button>
                  <button class="icon-btn icon-btn--del" title="Delete" @click="wb.deleteNote(note.id)">
                    <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor"
                         stroke-width="1.5" stroke-linecap="round">
                      <line x1="1" y1="1" x2="9" y2="9"/><line x1="9" y1="1" x2="1" y2="9"/>
                    </svg>
                  </button>
                </div>
              </div>

              <div v-if="wb.notes.value.length === 0 && !quickNoteOpen" class="notes-panel-empty">
                No notes yet
              </div>
            </div>

            <!-- Quick-add note -->
            <div v-if="quickNoteOpen" class="quick-note-form">
              <textarea
                ref="quickNoteInput"
                v-model="quickNoteBody"
                class="quick-note-input"
                placeholder="Observation, synthesis, open question…"
                rows="3"
                @keydown.ctrl.enter.prevent="submitQuickNote"
                @keydown.meta.enter.prevent="submitQuickNote"
                @keydown.escape="quickNoteOpen = false"
              ></textarea>
              <div class="quick-note-actions">
                <button class="btn-primary btn-sm" @click="submitQuickNote">Add note</button>
                <button class="btn-ghost btn-sm" @click="quickNoteOpen = false">Cancel</button>
                <span class="quick-note-hint">Ctrl+↵ to save</span>
              </div>
            </div>
            <button v-else class="add-note-btn" @click="openQuickNote">
              <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor"
                   stroke-width="1.5" stroke-linecap="round">
                <line x1="5.5" y1="1" x2="5.5" y2="10"/>
                <line x1="1" y1="5.5" x2="10" y2="5.5"/>
              </svg>
              Add note
            </button>
          </div>

        </div>

        <!-- ── Right: sidebar ───────────────────────────────────────────── -->
        <div class="wb-sidebar" :class="{ collapsed: sidebarCollapsed }">
          <div class="sidebar-header">
            <button class="sidebar-toggle" @click="sidebarCollapsed = !sidebarCollapsed" title="Toggle sidebar">
              <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor"
                   stroke-width="1.5" stroke-linecap="round">
                <line x1="1" y1="3" x2="12" y2="3"/>
                <line x1="1" y1="6.5" x2="12" y2="6.5"/>
                <line x1="1" y1="10" x2="12" y2="10"/>
              </svg>
            </button>
            <template v-if="!sidebarCollapsed">
              <div class="sidebar-tabs">
                <button class="sidebar-tab" :class="{ active: sidebarTab === 'literature' }"
                        @click="sidebarTab = 'literature'">Literature</button>
                <button class="sidebar-tab" :class="{ active: sidebarTab === 'notes' }"
                        @click="sidebarTab = 'notes'">My Notes</button>
              </div>
              <span class="sidebar-count">
                {{ sidebarTab === 'literature' ? sourceAnnotations.length : wb.notes.value.length }}
              </span>
            </template>
          </div>

          <template v-if="!sidebarCollapsed">

            <!-- Literature tab -->
            <template v-if="sidebarTab === 'literature'">
              <div class="sidebar-search-wrap">
                <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor"
                     stroke-width="1.5" stroke-linecap="round">
                  <circle cx="4.5" cy="4.5" r="3.5"/>
                  <line x1="7.5" y1="7.5" x2="10.5" y2="10.5"/>
                </svg>
                <input v-model="sidebarSearch" class="sidebar-search" placeholder="Filter…" />
              </div>
              <div class="sidebar-body">
                <div v-if="wb.sourceIds.value.length === 0" class="sidebar-empty">
                  Add sources above to see annotations here
                </div>
                <div v-else-if="sourceAnnotations.length === 0" class="sidebar-empty">
                  No annotations from these sources yet
                </div>
                <template v-else>
                  <div v-for="group in filteredGroups" :key="group.itemId" class="sidebar-group">
                    <div class="sidebar-group-header">
                      <span class="sidebar-group-title">{{ shortAuthors(group.anns[0].itemAuthors) }}{{ group.anns[0].itemYear ? ' ' + group.anns[0].itemYear : '' }}</span>
                      <span class="sidebar-group-count">{{ group.anns.length }}</span>
                    </div>
                    <div
                      v-for="a in group.anns"
                      :key="a.id"
                      class="sidebar-ann"
                      :class="{ 'is-dropped': droppedIds.has(a.id) }"
                    >
                      <div class="sidebar-stripe" :style="{ background: a.color }"></div>
                      <div class="sidebar-ann-body">
                        <div class="sidebar-ann-text">{{ annText(a) }}</div>
                        <div class="sidebar-ann-meta">p.&nbsp;{{ a.page }}</div>
                      </div>
                      <div v-if="droppedIds.has(a.id)" class="dropped-badge" title="Already in outline">✓</div>
                    </div>
                  </div>
                </template>
              </div>
            </template>

            <!-- My Notes tab -->
            <template v-else>
              <div class="sidebar-notes-toolbar">
                <button v-if="!newNoteOpen" class="new-note-btn" @click="newNoteOpen = true">
                  <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor"
                       stroke-width="1.6" stroke-linecap="round">
                    <line x1="5" y1="1" x2="5" y2="9"/>
                    <line x1="1" y1="5" x2="9" y2="5"/>
                  </svg>
                  New note
                </button>
              </div>

              <!-- New note form -->
              <div v-if="newNoteOpen" class="note-form">
                <textarea
                  v-model="newNoteBody"
                  class="note-form-body"
                  placeholder="Analysis summary, observation, finding…"
                  rows="3"
                  autofocus
                  @keydown.escape="newNoteOpen = false"
                ></textarea>
                <div class="note-form-code-row">
                  <input v-model="newNoteLang" class="note-form-lang" placeholder="lang (e.g. python)" maxlength="20" />
                  <textarea v-if="newNoteLang" v-model="newNoteCode" class="note-form-code" placeholder="code snippet…" rows="3"></textarea>
                </div>
                <div class="note-form-actions">
                  <button class="btn-primary btn-sm" @click="submitNewNote">Add note</button>
                  <button class="btn-ghost btn-sm" @click="newNoteOpen = false">Cancel</button>
                </div>
              </div>

              <div class="sidebar-body">
                <div v-if="wb.notes.value.length === 0 && !newNoteOpen" class="sidebar-empty">
                  No notes yet. Click "+ New note" to add one.
                </div>
                <template v-else>
                  <div
                    v-for="note in wb.notes.value"
                    :key="note.id"
                    class="sidebar-note"
                    :class="{ 'is-dropped': droppedNoteIds.has(note.id), 'is-editing': editingNoteId === note.id }"
                  >
                    <template v-if="editingNoteId === note.id">
                      <textarea v-model="editNoteBody" class="note-edit-body" rows="3" autofocus></textarea>
                      <div class="note-form-code-row">
                        <input v-model="editNoteLang" class="note-form-lang" placeholder="lang" maxlength="20" />
                        <textarea v-if="editNoteLang" v-model="editNoteCode" class="note-form-code" rows="3" placeholder="code…"></textarea>
                      </div>
                      <div class="note-form-actions">
                        <button class="btn-primary btn-sm" @click="commitEditNote">Save</button>
                        <button class="btn-ghost btn-sm" @click="editingNoteId = null">Cancel</button>
                      </div>
                    </template>
                    <template v-else>
                      <div class="sidebar-note-meta">{{ relativeTime(note.createdAt) }}</div>
                      <div class="sidebar-note-body" @dblclick="startEditNote(note)">{{ note.body }}</div>
                      <span v-if="note.code" class="sidebar-note-code-badge">
                        {{ note.language || 'code' }}
                      </span>
                      <div class="sidebar-note-actions">
                        <button class="icon-btn" title="Edit" @click="startEditNote(note)">
                          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor"
                               stroke-width="1.5" stroke-linecap="round">
                            <path d="M7 1L9 3L3.5 8.5H1.5V6.5L7 1Z"/>
                          </svg>
                        </button>
                        <button class="icon-btn icon-btn--del" title="Delete" @click="wb.deleteNote(note.id)">
                          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor"
                               stroke-width="1.5" stroke-linecap="round">
                            <line x1="1" y1="1" x2="9" y2="9"/><line x1="9" y1="1" x2="1" y2="9"/>
                          </svg>
                        </button>
                      </div>
                      <div v-if="droppedNoteIds.has(note.id)" class="dropped-badge">✓</div>
                    </template>
                  </div>
                </template>
              </div>
            </template>

          </template>
        </div>

      </div>
    </template>
  </div>
</template>

<style scoped>
.wb {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: var(--bg);
  position: relative;
}

/* ── Empty / new project ─────────────────────────────────────────────────── */

.wb-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.wb-empty-inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  text-align: center;
  color: var(--text-tertiary);
  max-width: 320px;
}

.wb-empty-inner svg { margin-bottom: 4px; }

.wb-empty-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
}

.wb-empty-sub {
  font-size: 12.5px;
  line-height: 1.5;
  color: var(--text-secondary);
}

.new-project-input {
  width: 100%;
  padding: 8px 12px;
  font-size: 13.5px;
  border: 1px solid var(--border-medium);
  border-radius: var(--radius);
  background: var(--surface-solid);
  color: var(--text);
  outline: none;
  font-family: var(--font-ui);
}

.new-project-input:focus { border-color: var(--accent); }

.new-project-actions {
  display: flex;
  gap: 8px;
}

/* ── Buttons ─────────────────────────────────────────────────────────────── */

.btn-primary {
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius-sm);
  padding: 6px 14px;
  font-size: 12.5px;
  font-family: var(--font-ui);
  font-weight: 500;
  cursor: pointer;
  transition: opacity var(--t);
}
.btn-primary:hover { opacity: 0.88; }
.btn-primary.btn-sm { padding: 4px 11px; font-size: 12px; }

.btn-ghost {
  background: none;
  color: var(--text-secondary);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  padding: 6px 14px;
  font-size: 12.5px;
  font-family: var(--font-ui);
  cursor: pointer;
  transition: background var(--t), border-color var(--t);
}
.btn-ghost:hover { background: var(--bg-chrome-active); }
.btn-ghost.btn-sm { padding: 4px 10px; font-size: 12px; }

.icon-btn {
  background: none;
  border: 1px solid transparent;
  width: 22px;
  height: 22px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  cursor: pointer;
  transition: background var(--t), color var(--t);
  flex-shrink: 0;
}
.icon-btn:hover { background: var(--bg-chrome-active); color: var(--text-secondary); }
.icon-btn:disabled { opacity: 0.3; cursor: default; pointer-events: none; }
.icon-btn--del:hover { background: rgba(200,50,50,0.08); color: #c83232; }

/* ── Header ──────────────────────────────────────────────────────────────── */

.wb-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-chrome);
  flex-shrink: 0;
  flex-wrap: wrap;
  row-gap: 6px;
}

.wb-header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
  flex-wrap: wrap;
}

.wb-header-right {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.header-sep {
  width: 1px;
  height: 18px;
  background: var(--border-medium);
  flex-shrink: 0;
}

/* Project name label */
.project-name-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex-shrink: 0;
}

/* Sources row */
.sources-row {
  display: flex;
  align-items: center;
  gap: 7px;
  flex-wrap: wrap;
}

.sources-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.source-chips {
  display: flex;
  align-items: center;
  gap: 5px;
  flex-wrap: wrap;
}

.source-chip {
  display: flex;
  align-items: center;
  gap: 4px;
  background: var(--surface-solid);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-pill);
  padding: 2px 8px 2px 10px;
  font-size: 11.5px;
  color: var(--text-secondary);
}

.chip-text { max-width: 120px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.chip-remove {
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  display: flex;
  align-items: center;
  padding: 0;
  transition: color var(--t);
}
.chip-remove:hover { color: #c83232; }

.chip-add {
  background: none;
  border: 1px dashed var(--border-medium);
  border-radius: var(--radius-pill);
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  cursor: pointer;
  transition: border-color var(--t), color var(--t), background var(--t);
}
.chip-add:hover { border-color: var(--accent); color: var(--accent); background: var(--accent-soft); }

/* Source picker dropdown */
.source-picker-wrap { position: relative; }

.source-picker {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  width: 300px;
  background: var(--surface-solid);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius);
  box-shadow: var(--shadow-float);
  z-index: 100;
  overflow: hidden;
}

.picker-search {
  width: 100%;
  padding: 9px 12px;
  border: none;
  border-bottom: 1px solid var(--border);
  font-size: 12.5px;
  font-family: var(--font-ui);
  background: var(--bg-chrome);
  color: var(--text);
  outline: none;
}

.picker-list { max-height: 220px; overflow-y: auto; }

.picker-empty {
  padding: 12px;
  font-size: 12px;
  color: var(--text-tertiary);
  text-align: center;
}

.picker-row {
  display: flex;
  flex-direction: column;
  gap: 2px;
  width: 100%;
  padding: 8px 12px;
  background: none;
  border: none;
  text-align: left;
  cursor: pointer;
  transition: background var(--t);
}
.picker-row:hover { background: var(--bg-chrome); }

.picker-title {
  font-size: 12.5px;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.picker-sub {
  font-size: 11px;
  color: var(--text-tertiary);
}

/* ── Body ────────────────────────────────────────────────────────────────── */

.wb-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* ── Sections (main panel) ───────────────────────────────────────────────── */

.wb-main {
  flex: 1;
  overflow-y: auto;
  padding: 16px 18px;
  min-width: 0;
}

.sections-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-width: 680px;
}

.section-card {
  background: var(--surface-solid);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: visible;
}

.section-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-chrome);
}

.section-num {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--bg-chrome-active);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 700;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.section-title {
  flex: 1;
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  cursor: text;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.section-title-input {
  flex: 1;
  font-size: 13px;
  font-weight: 600;
  font-family: var(--font-ui);
  color: var(--text);
  background: none;
  border: none;
  outline: none;
  min-width: 0;
}

.section-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  opacity: 0;
  transition: opacity var(--t);
}

.section-card:hover .section-actions { opacity: 1; }

/* Section body (dropped annotations) */
.section-body {
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-height: 48px;
}

/* {{ trigger input */
.section-trigger-wrap {
  position: relative;
}

.section-trigger-input {
  width: 100%;
  padding: 5px 9px;
  font-size: 12px;
  font-family: var(--font-ui);
  color: var(--text-secondary);
  background: transparent;
  border: 1px dashed var(--border);
  border-radius: var(--radius-sm);
  outline: none;
  transition: border-color var(--t), background var(--t);
}

.section-trigger-input::placeholder { color: var(--text-tertiary); font-size: 11.5px; }
.section-trigger-input:focus {
  border-color: var(--border-medium);
  background: var(--surface-solid);
  color: var(--text);
}

/* Suggestion dropdown */
.sug-list {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: var(--surface-solid);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius);
  box-shadow: var(--shadow-float);
  z-index: 50;
  overflow: hidden;
  max-height: 280px;
  overflow-y: auto;
}

.sug-row {
  display: flex;
  align-items: flex-start;
  gap: 0;
  padding: 7px 10px 7px 0;
  cursor: pointer;
  border-bottom: 1px solid var(--border);
  transition: background var(--t);
}
.sug-row:last-child { border-bottom: none; }
.sug-row:hover, .sug-row.sug-focused { background: var(--bg-chrome); }

.sug-stripe {
  width: 3px;
  align-self: stretch;
  flex-shrink: 0;
  opacity: 0.7;
  margin-right: 9px;
}

.sug-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }

.sug-text {
  font-size: 12px;
  color: var(--text);
  font-family: var(--font-doc);
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.sug-meta { font-size: 10.5px; color: var(--text-tertiary); }

.dropped-ann {
  display: flex;
  align-items: flex-start;
  gap: 0;
  border-radius: var(--radius-sm);
  overflow: hidden;
  background: var(--bg);
  border: 1px solid var(--border);
  transition: background var(--t);
}

.dropped-ann:hover { background: var(--bg-chrome); }

.dropped-stripe {
  width: 3px;
  align-self: stretch;
  flex-shrink: 0;
  opacity: 0.7;
}

.dropped-body {
  flex: 1;
  min-width: 0;
  padding: 6px 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.dropped-text {
  font-size: 12px;
  color: var(--text);
  font-family: var(--font-doc);
  line-height: 1.45;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.dropped-meta {
  font-size: 10.5px;
  color: var(--text-tertiary);
}

.dropped-note {
  width: 100%;
  margin-top: 5px;
  padding: 0;
  font-size: 12px;
  font-family: var(--font-ui);
  color: var(--text-secondary);
  background: transparent;
  border: none;
  border-top: 1px solid var(--border);
  padding-top: 5px;
  outline: none;
  resize: none;
  overflow: hidden;
  min-height: 20px;
  line-height: 1.5;
  display: block;
}

.dropped-note::placeholder { color: var(--text-tertiary); }
.dropped-note:focus { color: var(--text); }

.dropped-remove {
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 6px 7px;
  display: flex;
  align-items: flex-start;
  opacity: 0;
  transition: opacity var(--t), color var(--t);
}
.dropped-ann:hover .dropped-remove { opacity: 1; }
.dropped-remove:hover { color: #c83232; }

/* Add section */
.add-section-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: none;
  border: 1px dashed var(--border-medium);
  border-radius: var(--radius);
  padding: 9px 14px;
  font-size: 12.5px;
  font-family: var(--font-ui);
  color: var(--text-tertiary);
  cursor: pointer;
  transition: border-color var(--t), color var(--t), background var(--t);
  width: 100%;
  max-width: 680px;
}
.add-section-btn:hover { border-color: var(--accent); color: var(--accent); background: var(--accent-soft); }

.add-section-row { max-width: 680px; }
.add-section-input {
  width: 100%;
  padding: 9px 14px;
  font-size: 13px;
  font-weight: 600;
  font-family: var(--font-ui);
  background: var(--surface-solid);
  border: 1px solid var(--accent);
  border-radius: var(--radius);
  outline: none;
  color: var(--text);
}

/* ── Sidebar ─────────────────────────────────────────────────────────────── */

.wb-sidebar {
  width: 320px;
  flex-shrink: 0;
  border-left: 1px solid var(--border);
  background: var(--bg-chrome);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  transition: width var(--t);
}

.wb-sidebar.collapsed {
  width: 40px;
}

.sidebar-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 10px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.sidebar-toggle {
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  display: flex;
  align-items: center;
  padding: 3px;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
  transition: background var(--t), color var(--t);
}
.sidebar-toggle:hover { background: var(--bg-chrome-active); color: var(--text-secondary); }

.sidebar-title {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-tertiary);
  flex: 1;
}

.sidebar-count {
  font-size: 10.5px;
  font-weight: 600;
  color: var(--text-tertiary);
  background: var(--bg-chrome-active);
  border-radius: 9px;
  padding: 1px 6px;
}

.sidebar-search-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 10px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.sidebar-search-wrap svg { color: var(--text-tertiary); flex-shrink: 0; }

.sidebar-search {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  font-size: 12px;
  color: var(--text);
  font-family: var(--font-ui);
}
.sidebar-search::placeholder { color: var(--text-tertiary); }

.sidebar-body {
  flex: 1;
  overflow-y: auto;
  padding: 6px 0;
}

.sidebar-empty {
  padding: 16px 12px;
  font-size: 12px;
  color: var(--text-tertiary);
  line-height: 1.5;
  text-align: center;
}

/* Sidebar groups */
.sidebar-group { }

.sidebar-group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px 4px;
  position: sticky;
  top: 0;
  background: var(--bg-chrome);
  z-index: 1;
}

.sidebar-group-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
}

.sidebar-group-count {
  font-size: 10px;
  color: var(--text-tertiary);
}

/* Sidebar annotation row */
.sidebar-ann {
  display: flex;
  align-items: flex-start;
  gap: 0;
  padding: 5px 10px 5px 0;
  cursor: grab;
  transition: background var(--t);
  border-bottom: 1px solid var(--border);
  user-select: none;
}

.sidebar-ann:hover { background: var(--bg-chrome-active); }
.sidebar-ann.is-dropped { opacity: 0.45; }
.sidebar-ann:active { cursor: grabbing; }

.sidebar-stripe {
  width: 3px;
  align-self: stretch;
  flex-shrink: 0;
  opacity: 0.7;
  margin-right: 8px;
}

.sidebar-ann-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.sidebar-ann-text {
  font-size: 11.5px;
  color: var(--text);
  font-family: var(--font-doc);
  line-height: 1.45;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.sidebar-ann-meta {
  font-size: 10.5px;
  color: var(--text-tertiary);
}

.dropped-badge {
  font-size: 9px;
  color: var(--text-tertiary);
  flex-shrink: 0;
  align-self: center;
  padding: 0 4px;
}

/* ── Sidebar tabs ─────────────────────────────────────────────────────────── */

.sidebar-tabs {
  display: flex;
  gap: 2px;
  flex: 1;
}

.sidebar-tab {
  background: none;
  border: none;
  font-size: 11px;
  font-weight: 600;
  font-family: var(--font-ui);
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 3px 8px;
  border-radius: var(--radius-sm);
  transition: background var(--t), color var(--t);
}

.sidebar-tab.active {
  color: var(--text);
  background: var(--bg-chrome-active);
}

.sidebar-tab:hover:not(.active) {
  color: var(--text-secondary);
  background: var(--bg-chrome-active);
}

/* ── My Notes toolbar ────────────────────────────────────────────────────── */

.sidebar-notes-toolbar {
  padding: 6px 10px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.new-note-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  background: none;
  border: 1px dashed var(--border-medium);
  border-radius: var(--radius-sm);
  padding: 4px 10px;
  font-size: 11.5px;
  font-family: var(--font-ui);
  color: var(--text-tertiary);
  cursor: pointer;
  transition: border-color var(--t), color var(--t), background var(--t);
  width: 100%;
}

.new-note-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--accent-soft);
}

/* ── Note form ───────────────────────────────────────────────────────────── */

.note-form {
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex-shrink: 0;
}

.note-form-body {
  width: 100%;
  padding: 6px 8px;
  font-size: 12px;
  font-family: var(--font-ui);
  color: var(--text);
  background: var(--surface-solid);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  outline: none;
  resize: none;
  line-height: 1.5;
}

.note-form-body:focus { border-color: var(--accent); }

.note-form-code-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.note-form-lang {
  padding: 4px 8px;
  font-size: 11.5px;
  font-family: var(--font-ui);
  color: var(--text-secondary);
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  outline: none;
  width: 120px;
}

.note-form-lang:focus { border-color: var(--accent); }

.note-form-code {
  width: 100%;
  padding: 5px 8px;
  font-size: 11px;
  font-family: 'Menlo', 'Consolas', monospace;
  color: var(--text);
  background: var(--bg-document);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  outline: none;
  resize: none;
  line-height: 1.5;
}

.note-form-actions {
  display: flex;
  gap: 6px;
}

/* ── Sidebar note rows ───────────────────────────────────────────────────── */

.sidebar-note {
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 3px;
  position: relative;
  transition: background var(--t);
}

.sidebar-note:hover { background: var(--bg-chrome-active); }
.sidebar-note.is-dropped { opacity: 0.5; }

.sidebar-note-meta {
  font-size: 10px;
  color: var(--text-tertiary);
  letter-spacing: 0.02em;
}

.sidebar-note-body {
  font-size: 12px;
  color: var(--text);
  line-height: 1.45;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
  cursor: text;
}

.sidebar-note-code-badge {
  display: inline-flex;
  align-items: center;
  background: var(--bg-document);
  border: 1px solid var(--border);
  border-radius: 3px;
  padding: 1px 5px;
  font-size: 10px;
  font-family: 'Menlo', 'Consolas', monospace;
  color: var(--text-tertiary);
  align-self: flex-start;
}

.sidebar-note-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity var(--t);
  position: absolute;
  top: 6px;
  right: 6px;
}

.sidebar-note:hover .sidebar-note-actions { opacity: 1; }

.note-edit-body {
  width: 100%;
  padding: 5px 7px;
  font-size: 12px;
  font-family: var(--font-ui);
  color: var(--text);
  background: var(--surface-solid);
  border: 1px solid var(--accent);
  border-radius: var(--radius-sm);
  outline: none;
  resize: none;
  line-height: 1.5;
}

/* ── Dropped note block (in section cards) ───────────────────────────────── */

.dropped-note-block {
  display: flex;
  align-items: flex-start;
  gap: 0;
  border-radius: var(--radius-sm);
  overflow: hidden;
  background: var(--bg);
  border: 1px solid var(--border);
  transition: background var(--t);
}

.dropped-note-block:hover { background: var(--bg-chrome); }

.dropped-note-stripe {
  width: 3px;
  align-self: stretch;
  flex-shrink: 0;
  background: var(--text-tertiary);
  opacity: 0.35;
}

.dropped-note-ts {
  font-size: 10px;
  color: var(--text-tertiary);
  margin-bottom: 2px;
}

.sug-stripe--note {
  width: 3px;
  align-self: stretch;
  flex-shrink: 0;
  background: var(--text-tertiary);
  opacity: 0.35;
  margin-right: 9px;
}

/* ── Notes panel (below outline) ─────────────────────────────────────────── */

.notes-panel {
  max-width: 680px;
  margin-top: 24px;
  padding-top: 20px;
  border-top: 1px solid var(--border-medium);
}

.notes-panel-head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.notes-panel-label {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-tertiary);
}

.notes-panel-count {
  font-size: 10.5px;
  font-weight: 600;
  color: var(--text-tertiary);
  background: var(--bg-chrome-active);
  border-radius: 9px;
  padding: 1px 6px;
}

.notes-panel-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 8px;
}

.notes-panel-empty {
  font-size: 12px;
  color: var(--text-tertiary);
  padding: 4px 0 8px;
}

.note-inline-card {
  background: var(--surface-solid);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 9px 12px;
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 3px;
  transition: border-color var(--t), box-shadow var(--t);
}

.note-inline-card:hover {
  border-color: var(--border-medium);
  box-shadow: var(--shadow-xs);
}

.note-inline-card.is-dropped {
  opacity: 0.5;
}

.note-inline-ts {
  font-size: 10px;
  color: var(--text-tertiary);
}

.note-inline-body {
  font-size: 12.5px;
  color: var(--text);
  line-height: 1.5;
}

.note-inline-code-badge {
  display: inline-flex;
  align-items: center;
  background: var(--bg-document);
  border: 1px solid var(--border);
  border-radius: 3px;
  padding: 1px 5px;
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--text-tertiary);
  align-self: flex-start;
}

.note-inline-actions {
  display: flex;
  gap: 2px;
  opacity: 0;
  transition: opacity var(--t);
  position: absolute;
  top: 6px;
  right: 6px;
}

.note-inline-card:hover .note-inline-actions {
  opacity: 1;
}

.quick-note-form {
  display: flex;
  flex-direction: column;
  gap: 6px;
  background: var(--surface-solid);
  border: 1.5px solid var(--accent);
  border-radius: var(--radius);
  padding: 10px 12px;
  margin-bottom: 4px;
}

.quick-note-input {
  width: 100%;
  padding: 0;
  font-size: 12.5px;
  font-family: var(--font-ui);
  color: var(--text);
  background: transparent;
  border: none;
  outline: none;
  resize: none;
  line-height: 1.5;
}

.quick-note-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.quick-note-hint {
  font-size: 10.5px;
  color: var(--text-tertiary);
  margin-left: auto;
}

.add-note-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: none;
  border: 1px dashed var(--border-medium);
  border-radius: var(--radius);
  padding: 7px 14px;
  font-size: 12px;
  font-family: var(--font-ui);
  color: var(--text-tertiary);
  cursor: pointer;
  transition: border-color var(--t), color var(--t), background var(--t);
  width: 100%;
}

.add-note-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
  background: var(--accent-soft);
}

/* Code toggle + block */
.code-toggle {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: var(--bg-document);
  border: 1px solid var(--border);
  border-radius: 3px;
  padding: 2px 6px;
  font-size: 10.5px;
  font-family: var(--font-ui);
  color: var(--text-secondary);
  cursor: pointer;
  margin-top: 4px;
  transition: background var(--t);
}

.code-toggle:hover { background: var(--bg-chrome-active); }

.code-lang {
  font-family: 'Menlo', 'Consolas', monospace;
  font-size: 10px;
  color: var(--text-tertiary);
}

.code-block {
  margin-top: 4px;
  padding: 7px 9px;
  background: var(--bg-document);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-family: 'Menlo', 'Consolas', monospace;
  line-height: 1.5;
  overflow-x: auto;
  white-space: pre;
  color: var(--text);
}
</style>
