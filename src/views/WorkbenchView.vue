<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useWorkbench, type ProjectSection } from '../composables/useWorkbench'
import { useAllAnnotations, type AnnotationWithSource } from '../composables/useAnnotations'

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

// ── New project form ──────────────────────────────────────────────────────────

const newProjectOpen = ref(false)
const newProjectName = ref('')

async function submitNewProject() {
  const name = newProjectName.value.trim()
  if (!name) return
  await wb.createProject(name)
  newProjectName.value = ''
  newProjectOpen.value = false
}

// ── Project rename ────────────────────────────────────────────────────────────

const renamingProject = ref(false)
const renameDraft     = ref('')

function startRenameProject() {
  renameDraft.value = wb.currentProject.value?.name ?? ''
  renamingProject.value = true
  nextTick(() => (document.querySelector('.rename-input') as HTMLInputElement | null)?.select())
}

async function commitRenameProject() {
  const name = renameDraft.value.trim()
  if (name && wb.currentId.value) await wb.renameProject(wb.currentId.value, name)
  renamingProject.value = false
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

// ── {{ trigger: annotation picker per section ────────────────────────────────

const sectionQuery    = ref<Record<number, string>>({})
const activeSectionId = ref<number | null>(null)
const focusedIdx      = ref(0)

const suggestions = computed(() => {
  if (activeSectionId.value === null) return []
  const raw = sectionQuery.value[activeSectionId.value] ?? ''
  const trigger = raw.lastIndexOf('{{')
  if (trigger === -1) return []
  const search = raw.slice(trigger + 2).toLowerCase().trim()
  return sourceAnnotations.value
    .filter(a => !droppedIds.value.has(a.id))
    .filter(a =>
      !search ||
      (a.selectedText?.toLowerCase().includes(search)) ||
      (a.noteText?.toLowerCase().includes(search)) ||
      (a.itemTitle?.toLowerCase().includes(search)) ||
      (a.itemAuthors?.toLowerCase().includes(search))
    )
    .slice(0, 8)
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
  // small delay so mousedown on suggestion fires first
  setTimeout(() => { activeSectionId.value = null }, 180)
}

function clearQuery(sectionId: number) {
  sectionQuery.value = { ...sectionQuery.value, [sectionId]: '' }
  activeSectionId.value = null
}

function suggestDown() { focusedIdx.value = Math.min(focusedIdx.value + 1, suggestions.value.length - 1) }
function suggestUp()   { focusedIdx.value = Math.max(focusedIdx.value - 1, 0) }

async function selectSuggestion(sectionId: number, annId: number) {
  await wb.dropAnnotation(sectionId, annId)
  clearQuery(sectionId)
}

async function selectFocused(sectionId: number) {
  const a = suggestions.value[focusedIdx.value]
  if (a) await selectSuggestion(sectionId, a.id)
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

// ── Sidebar search ────────────────────────────────────────────────────────────

const sidebarSearch = ref('')
const sidebarCollapsed = ref(false)

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

function autoResize(e: Event) {
  const el = e.target as HTMLTextAreaElement
  el.style.height = 'auto'
  el.style.height = el.scrollHeight + 'px'
}
</script>

<template>
  <div class="wb" @click.self="closePicker">

    <!-- ── Empty: no projects ───────────────────────────────────────────────── -->
    <div v-if="!wb.loading.value && wb.projects.value.length === 0 && !newProjectOpen"
         class="wb-empty">
      <div class="wb-empty-inner">
        <svg width="32" height="32" viewBox="0 0 32 32" fill="none" stroke="currentColor"
             stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="4" y="6" width="24" height="20" rx="2"/>
          <line x1="10" y1="13" x2="22" y2="13"/>
          <line x1="10" y1="18" x2="17" y2="18"/>
        </svg>
        <p class="wb-empty-title">No workbench projects yet</p>
        <p class="wb-empty-sub">A project groups your source set and rough outline for a paper.</p>
        <button class="btn-primary" @click="newProjectOpen = true">New project</button>
      </div>
    </div>

    <!-- ── New project input overlay ────────────────────────────────────────── -->
    <div v-if="newProjectOpen" class="wb-empty">
      <div class="wb-empty-inner">
        <p class="wb-empty-title">Name your project</p>
        <input
          v-model="newProjectName"
          class="new-project-input"
          placeholder="e.g. Allergen Labelling Study"
          maxlength="80"
          autofocus
          @keydown.enter="submitNewProject"
          @keydown.escape="newProjectOpen = false"
        />
        <div class="new-project-actions">
          <button class="btn-primary" @click="submitNewProject">Create</button>
          <button class="btn-ghost" @click="newProjectOpen = false">Cancel</button>
        </div>
      </div>
    </div>

    <!-- ── Main workbench ────────────────────────────────────────────────────── -->
    <template v-else-if="wb.currentProject.value">

      <!-- Header bar -->
      <div class="wb-header">
        <div class="wb-header-left">
          <!-- Project switcher -->
          <div class="project-select-wrap">
            <select
              class="project-select"
              :value="wb.currentId.value"
              @change="wb.selectProject(Number(($event.target as HTMLSelectElement).value))"
            >
              <option v-for="p in wb.projects.value" :key="p.id" :value="p.id">{{ p.name }}</option>
            </select>
            <svg class="select-chevron" width="10" height="10" viewBox="0 0 10 10" fill="none"
                 stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
              <path d="M2 4l3 3 3-3"/>
            </svg>
          </div>

          <!-- Rename project -->
          <template v-if="renamingProject">
            <input
              v-model="renameDraft"
              class="rename-input"
              @keydown.enter="commitRenameProject"
              @keydown.escape="renamingProject = false"
              @blur="commitRenameProject"
            />
          </template>
          <button v-else class="icon-btn" title="Rename project" @click="startRenameProject">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor"
                 stroke-width="1.5" stroke-linecap="round">
              <path d="M8.5 1.5L10.5 3.5L4 10H2V8L8.5 1.5Z"/>
            </svg>
          </button>

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
          <button class="btn-ghost btn-sm"
                  @click="wb.deleteProject(wb.currentId.value!)"
                  title="Delete project">
            Delete project
          </button>
          <button class="btn-ghost btn-sm" @click="newProjectOpen = true">
            New project
          </button>
          <button class="btn-primary btn-sm" @click="router.push('/write')">
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

              <!-- Dropped annotations + {{ trigger -->
              <div class="section-body">
                <div
                  v-for="entry in sec.annotations"
                  :key="entry.annotationId"
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

                <!-- {{ annotation trigger -->
                <div class="section-trigger-wrap">
                  <input
                    :value="sectionQuery[sec.id] ?? ''"
                    class="section-trigger-input"
                    :placeholder="sec.annotations.length === 0 ? 'Type {{ to add an annotation…' : '{{'"
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
                      v-for="(a, i) in suggestions"
                      :key="a.id"
                      class="sug-row"
                      :class="{ 'sug-focused': i === focusedIdx }"
                      @mousedown.prevent="selectSuggestion(sec.id, a.id)"
                    >
                      <div class="sug-stripe" :style="{ background: a.color }"></div>
                      <div class="sug-body">
                        <div class="sug-text">{{ annText(a) }}</div>
                        <div class="sug-meta">{{ shortAuthors(a.itemAuthors) }}{{ a.itemYear ? ' ' + a.itemYear : '' }} · p.{{ a.page }}</div>
                      </div>
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
        </div>

        <!-- ── Right: annotation sidebar ─────────────────────────────────── -->
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
              <span class="sidebar-title">Annotations</span>
              <span class="sidebar-count">{{ sourceAnnotations.length }}</span>
            </template>
          </div>

          <template v-if="!sidebarCollapsed">
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

/* Project select */
.project-select-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.project-select {
  appearance: none;
  background: none;
  border: none;
  font-size: 13px;
  font-weight: 600;
  font-family: var(--font-ui);
  color: var(--text);
  cursor: pointer;
  outline: none;
  padding-right: 18px;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.select-chevron {
  position: absolute;
  right: 2px;
  color: var(--text-tertiary);
  pointer-events: none;
}

.rename-input {
  font-size: 13px;
  font-weight: 600;
  font-family: var(--font-ui);
  background: var(--surface-solid);
  border: 1px solid var(--accent);
  border-radius: var(--radius-sm);
  padding: 2px 7px;
  outline: none;
  color: var(--text);
  width: 180px;
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
</style>
