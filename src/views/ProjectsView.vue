<script setup lang="ts">
import { ref, onMounted, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { useWorkbench } from '../composables/useWorkbench'

const router = useRouter()
const {
  projects, currentId, loading,
  loadProjects, createProject, deleteProject,
  renameProject, updateProjectMeta, selectProject,
} = useWorkbench()

const showCreate   = ref(false)
const createName   = ref('')
const createInput  = ref<HTMLInputElement | null>(null)

const renamingId   = ref<number | null>(null)
const renameValue  = ref('')
const renameInput  = ref<HTMLInputElement | null>(null)

onMounted(() => { loadProjects() })

async function openCreate() {
  showCreate.value = true
  createName.value = ''
  await nextTick()
  createInput.value?.focus()
}

async function submitCreate() {
  const name = createName.value.trim()
  showCreate.value = false
  if (!name) return
  await createProject(name)
  // createProject already calls selectProject internally
  router.push('/write')
}

function cancelCreate() {
  showCreate.value = false
  createName.value = ''
}

async function openProject(id: number) {
  await selectProject(id)
  router.push('/write')
}

async function startRename(id: number, name: string, e: Event) {
  e.stopPropagation()
  renamingId.value = id
  renameValue.value = name
  await nextTick()
  renameInput.value?.select()
}

async function submitRename(id: number) {
  const name = renameValue.value.trim()
  if (name) await renameProject(id, name)
  renamingId.value = null
}

function cancelRename() {
  renamingId.value = null
}

async function confirmDelete(id: number, name: string, e: Event) {
  e.stopPropagation()
  if (!confirm(`Delete "${name}"?\n\nThe project folder will remain on disk.`)) return
  await deleteProject(id)
}

const STATUS_CYCLE = ['draft', 'in-progress', 'submitted', 'published'] as const
type Status = typeof STATUS_CYCLE[number]

async function cycleStatus(id: number, current: string, e: Event) {
  e.stopPropagation()
  const idx  = STATUS_CYCLE.indexOf(current as Status)
  const next = STATUS_CYCLE[(idx + 1) % STATUS_CYCLE.length]
  await updateProjectMeta(id, next)
}

function statusLabel(s: string) {
  return s === 'in-progress' ? 'in progress' : s
}

function relativeDate(ts: number) {
  const diff = Date.now() / 1000 - ts
  if (diff < 3600)      return `${Math.floor(diff / 60)}m ago`
  if (diff < 86400)     return `${Math.floor(diff / 3600)}h ago`
  if (diff < 86400 * 7) return `${Math.floor(diff / 86400)}d ago`
  return new Date(ts * 1000).toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' })
}
</script>

<template>
  <div class="projects-view">
    <div class="pv-header">
      <h1 class="pv-title">Projects</h1>
      <button class="pv-new-btn" @click="openCreate">
        <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
          <line x1="6.5" y1="1" x2="6.5" y2="12"/>
          <line x1="1" y1="6.5" x2="12" y2="6.5"/>
        </svg>
        New Project
      </button>
    </div>

    <!-- Create input row -->
    <div v-if="showCreate" class="pv-create-row">
      <input
        ref="createInput"
        v-model="createName"
        class="pv-create-input"
        placeholder="Project name…"
        @keydown.enter="submitCreate"
        @keydown.escape="cancelCreate"
        @blur="submitCreate"
      />
      <span class="pv-create-hint">↵ to create, Esc to cancel</span>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="pv-empty">Loading…</div>

    <!-- Empty state -->
    <div v-else-if="!loading && projects.length === 0 && !showCreate" class="pv-empty">
      <svg width="40" height="40" viewBox="0 0 40 40" fill="none" stroke="currentColor" stroke-width="1.2" opacity="0.3">
        <rect x="6" y="10" width="28" height="22" rx="3"/>
        <path d="M6 16h28"/>
        <path d="M13 23h14M13 27h9"/>
        <rect x="2" y="6" width="28" height="22" rx="3" stroke-dasharray="3 2"/>
      </svg>
      <p>No projects yet</p>
      <button class="pv-empty-btn" @click="openCreate">Create your first project</button>
    </div>

    <!-- Project list -->
    <ul v-else class="pv-list">
      <li
        v-for="p in projects"
        :key="p.id"
        class="pv-row"
        :class="{ 'pv-row--active': p.id === currentId }"
        @click="openProject(p.id)"
      >
        <!-- Active indicator -->
        <div class="pv-row-rail" :class="{ 'pv-row-rail--on': p.id === currentId }"></div>

        <div class="pv-row-body">
          <div class="pv-row-top">
            <!-- Name / rename -->
            <template v-if="renamingId === p.id">
              <input
                ref="renameInput"
                v-model="renameValue"
                class="pv-rename-input"
                @keydown.enter="submitRename(p.id)"
                @keydown.escape="cancelRename"
                @blur="submitRename(p.id)"
                @click.stop
              />
            </template>
            <span v-else class="pv-name">{{ p.name }}</span>

            <!-- Status badge -->
            <button
              class="pv-status"
              :class="`pv-status--${p.status}`"
              @click="cycleStatus(p.id, p.status, $event)"
              :title="`Status: ${statusLabel(p.status)} — click to cycle`"
            >{{ statusLabel(p.status) }}</button>
          </div>

          <div class="pv-row-meta">
            <span v-if="p.targetVenue" class="pv-venue">{{ p.targetVenue }}</span>
            <span class="pv-date">Created {{ relativeDate(p.createdAt) }}</span>
            <span class="pv-folder">{{ p.folderPath }}</span>
          </div>
        </div>

        <!-- Row actions -->
        <div class="pv-row-actions" @click.stop>
          <button class="pv-action-btn" title="Rename" @click="startRename(p.id, p.name, $event)">
            <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M9 2l2 2-6 6H3V8l6-6z"/>
              <line x1="1" y1="12" x2="10" y2="12"/>
            </svg>
          </button>
          <button class="pv-action-btn pv-action-btn--danger" title="Delete" @click="confirmDelete(p.id, p.name, $event)">
            <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="2,3.5 11,3.5"/>
              <path d="M4.5 3.5V2h4v1.5"/>
              <rect x="3" y="3.5" width="7" height="8" rx="1"/>
              <line x1="5.5" y1="6" x2="5.5" y2="9"/>
              <line x1="7.5" y1="6" x2="7.5" y2="9"/>
            </svg>
          </button>
        </div>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.projects-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--bg-document);
  overflow: hidden;
  padding: 32px 40px 24px;
  gap: 0;
}

/* ── Header ─────────────────────────────────────────────────────────────── */

.pv-header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  margin-bottom: 24px;
  flex-shrink: 0;
}

.pv-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text);
  letter-spacing: -0.3px;
}

.pv-new-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius-sm);
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;
  transition: opacity var(--t);
}
.pv-new-btn:hover { opacity: 0.88; }

/* ── Create row ─────────────────────────────────────────────────────────── */

.pv-create-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
  padding: 10px 14px;
  background: var(--surface-solid);
  border-radius: var(--radius);
  border: 1.5px solid var(--accent);
  box-shadow: var(--shadow-xs);
  flex-shrink: 0;
}

.pv-create-input {
  flex: 1;
  border: none;
  outline: none;
  font-size: 14px;
  font-weight: 500;
  color: var(--text);
  background: transparent;
  font-family: var(--font-ui);
}

.pv-create-hint {
  font-size: 11px;
  color: var(--text-tertiary);
  white-space: nowrap;
}

/* ── Empty state ────────────────────────────────────────────────────────── */

.pv-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--text-tertiary);
  font-size: 13px;
}

.pv-empty-btn {
  margin-top: 4px;
  padding: 7px 16px;
  background: var(--surface-solid);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background var(--t);
  font-family: var(--font-ui);
}
.pv-empty-btn:hover { background: var(--bg-chrome-active); }

/* ── List ───────────────────────────────────────────────────────────────── */

.pv-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 6px;
  overflow-y: auto;
  flex: 1;
  padding-right: 2px;
}

.pv-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px 12px 0;
  background: var(--surface-solid);
  border-radius: var(--radius);
  border: 1px solid var(--border);
  cursor: pointer;
  transition: box-shadow var(--t), border-color var(--t);
  position: relative;
  overflow: hidden;
}

.pv-row:hover {
  box-shadow: var(--shadow-sm);
  border-color: var(--border-medium);
}

.pv-row--active {
  border-color: rgba(10, 95, 191, 0.2);
  background: #fafcff;
}

/* Left rail for active project */
.pv-row-rail {
  width: 3px;
  align-self: stretch;
  flex-shrink: 0;
  border-radius: 0 2px 2px 0;
  background: transparent;
  transition: background var(--t);
  margin-left: 0;
}
.pv-row-rail--on {
  background: var(--accent);
}

.pv-row-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.pv-row-top {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pv-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  min-width: 0;
}

.pv-rename-input {
  flex: 1;
  border: none;
  outline: none;
  font-size: 14px;
  font-weight: 500;
  color: var(--text);
  background: transparent;
  font-family: var(--font-ui);
  border-bottom: 1.5px solid var(--accent);
}

/* Status badge */
.pv-status {
  flex-shrink: 0;
  border: none;
  border-radius: var(--radius-pill);
  font-size: 10.5px;
  font-weight: 500;
  padding: 2px 8px;
  cursor: pointer;
  font-family: var(--font-ui);
  letter-spacing: 0.2px;
  transition: opacity var(--t);
}
.pv-status:hover { opacity: 0.75; }

.pv-status--draft       { background: rgba(0,0,0,0.06); color: var(--text-tertiary); }
.pv-status--in-progress { background: var(--accent-soft); color: var(--accent); }
.pv-status--submitted   { background: rgba(232,101,10,0.1); color: var(--accent-orange); }
.pv-status--published   { background: rgba(22,150,63,0.1); color: var(--accent-green); }

/* Meta row */
.pv-row-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.pv-venue {
  font-size: 12px;
  color: var(--text-secondary);
  font-style: italic;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 280px;
}

.pv-date {
  font-size: 11.5px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.pv-folder {
  font-size: 10.5px;
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  opacity: 0.6;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 260px;
}

/* Row actions (hidden until hover) */
.pv-row-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity var(--t);
  padding-right: 4px;
}

.pv-row:hover .pv-row-actions {
  opacity: 1;
}

.pv-action-btn {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  border-radius: var(--radius-xs);
  color: var(--text-tertiary);
  cursor: pointer;
  transition: background var(--t), color var(--t);
}
.pv-action-btn:hover {
  background: var(--bg-chrome-active);
  color: var(--text-secondary);
}
.pv-action-btn--danger:hover {
  background: rgba(220, 38, 38, 0.08);
  color: #dc2626;
}
</style>
