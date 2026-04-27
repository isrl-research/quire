<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAllAnnotations, type AnnotationWithSource } from '../composables/useAnnotations'

const router = useRouter()
const { allAnnotations, allAnnotationsLoading, loadAll, deleteAndRefresh, updateNoteAndRefresh } = useAllAnnotations()

onMounted(loadAll)

// ── Search ────────────────────────────────────────────────────────────────────

const searchTerm = ref('')
const groupBySource = ref(true)

const filtered = computed(() => {
  const q = searchTerm.value.trim().toLowerCase()
  if (!q) return allAnnotations.value
  return allAnnotations.value.filter(a =>
    (a.selectedText?.toLowerCase().includes(q)) ||
    (a.noteText?.toLowerCase().includes(q)) ||
    (a.itemTitle?.toLowerCase().includes(q)) ||
    (a.itemAuthors?.toLowerCase().includes(q))
  )
})

// ── Grouped view ──────────────────────────────────────────────────────────────

interface Group {
  key: string
  label: string
  sub: string
  annotations: AnnotationWithSource[]
}

const groups = computed<Group[]>(() => {
  const map = new Map<string, Group>()
  for (const a of filtered.value) {
    const k = String(a.itemId)
    if (!map.has(k)) {
      const authors = shortAuthors(a.itemAuthors)
      map.set(k, {
        key: k,
        label: a.itemTitle ?? a.itemKey,
        sub: [authors, a.itemYear].filter(Boolean).join(', '),
        annotations: [],
      })
    }
    map.get(k)!.annotations.push(a)
  }
  return [...map.values()]
})

// ── Note editing ──────────────────────────────────────────────────────────────

const editingId = ref<number | null>(null)
const editDraft = ref('')

function startEdit(a: AnnotationWithSource) {
  editingId.value = a.id
  editDraft.value = a.noteText ?? ''
}

async function commitEdit(id: number) {
  await updateNoteAndRefresh(id, editDraft.value)
  editingId.value = null
}

function cancelEdit() {
  editingId.value = null
}

// ── Navigation ────────────────────────────────────────────────────────────────

function openInPdf(a: AnnotationWithSource) {
  router.push({
    name: 'pdf',
    query: { id: a.attachmentId, itemId: a.itemId, name: a.attachmentFileName, page: a.page },
  })
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function shortAuthors(authors?: string): string {
  if (!authors) return ''
  const parts = authors.split(/[,;&]|and\s/i).map(s => s.trim()).filter(Boolean)
  if (parts.length === 0) return ''
  const last = parts[0].split(/\s+/).pop() ?? parts[0]
  return parts.length > 2 ? `${last} et al.` : parts.length === 2
    ? `${last} & ${(parts[1].split(/\s+/).pop() ?? parts[1])}`
    : last
}
</script>

<template>
  <div class="ann-view">
    <!-- Toolbar -->
    <div class="ann-toolbar">
      <div class="search-wrap">
        <svg class="search-icon" width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
          <circle cx="5.5" cy="5.5" r="4"/>
          <line x1="8.5" y1="8.5" x2="12" y2="12"/>
        </svg>
        <input
          v-model="searchTerm"
          class="search-input"
          placeholder="Search annotations…"
          spellcheck="false"
        />
        <button v-if="searchTerm" class="search-clear" @click="searchTerm = ''">
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <line x1="1" y1="1" x2="9" y2="9"/><line x1="9" y1="1" x2="1" y2="9"/>
          </svg>
        </button>
      </div>

      <div class="toolbar-right">
        <span class="ann-count">{{ filtered.length }} annotation{{ filtered.length !== 1 ? 's' : '' }}</span>
        <button
          class="group-toggle"
          :class="{ active: groupBySource }"
          title="Group by source"
          @click="groupBySource = !groupBySource"
        >
          <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            <line x1="1" y1="3" x2="12" y2="3"/>
            <line x1="3" y1="6.5" x2="12" y2="6.5"/>
            <line x1="5" y1="10" x2="12" y2="10"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="allAnnotationsLoading" class="empty-state">
      <span class="empty-label">Loading…</span>
    </div>
    <div v-else-if="filtered.length === 0" class="empty-state">
      <svg width="28" height="28" viewBox="0 0 28 28" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round">
        <path d="M6 4h16a2 2 0 012 2v14a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2z"/>
        <line x1="9" y1="10" x2="19" y2="10"/>
        <line x1="9" y1="14" x2="15" y2="14"/>
      </svg>
      <span class="empty-label">{{ searchTerm ? 'No matches' : 'No annotations yet — highlight text in a PDF to create one' }}</span>
    </div>

    <!-- Grouped list -->
    <div v-else-if="groupBySource" class="ann-list">
      <div v-for="group in groups" :key="group.key" class="group">
        <div class="group-header">
          <span class="group-title">{{ group.label }}</span>
          <span class="group-sub">{{ group.sub }}</span>
          <span class="group-count">{{ group.annotations.length }}</span>
        </div>
        <div
          v-for="a in group.annotations"
          :key="a.id"
          class="ann-row"
        >
          <div class="color-stripe" :style="{ background: a.color }"></div>
          <div class="ann-body">
            <div class="ann-text" @click="openInPdf(a)">
              <span v-if="a.selectedText" class="ann-quote">"{{ a.selectedText }}"</span>
              <span v-else class="ann-quote ann-quote--empty">(no text selected)</span>
            </div>
            <div class="ann-meta">
              <span class="ann-page">p.&nbsp;{{ a.page }}</span>
              <span class="meta-sep">·</span>
              <span class="ann-file">{{ a.attachmentFileName }}</span>
            </div>
            <template v-if="editingId === a.id">
              <textarea
                v-model="editDraft"
                class="note-edit"
                rows="2"
                placeholder="Add a note…"
                @keydown.enter.ctrl="commitEdit(a.id)"
                @keydown.escape="cancelEdit"
              ></textarea>
              <div class="note-edit-actions">
                <button class="note-save" @click="commitEdit(a.id)">Save</button>
                <button class="note-cancel" @click="cancelEdit">Cancel</button>
              </div>
            </template>
            <div v-else-if="a.noteText" class="ann-note" @click="startEdit(a)">{{ a.noteText }}</div>
            <button v-else class="note-add-btn" @click="startEdit(a)">+ note</button>
          </div>
          <div class="ann-actions">
            <button class="action-btn" title="Open in PDF" @click="openInPdf(a)">
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                <path d="M2 2h8v8"/><path d="M2 10L10 2"/>
              </svg>
            </button>
            <button class="action-btn action-del" title="Delete annotation" @click="deleteAndRefresh(a.id)">
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                <path d="M2 3h8M5 3V2h2v1M4 3v6.5a.5.5 0 00.5.5h3a.5.5 0 00.5-.5V3"/>
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Flat list -->
    <div v-else class="ann-list">
      <div
        v-for="a in filtered"
        :key="a.id"
        class="ann-row"
      >
        <div class="color-stripe" :style="{ background: a.color }"></div>
        <div class="ann-body">
          <div class="ann-text" @click="openInPdf(a)">
            <span v-if="a.selectedText" class="ann-quote">"{{ a.selectedText }}"</span>
            <span v-else class="ann-quote ann-quote--empty">(no text selected)</span>
          </div>
          <div class="ann-meta">
            <span class="ann-source">{{ shortAuthors(a.itemAuthors) }}{{ a.itemYear ? ' ' + a.itemYear : '' }}</span>
            <span class="meta-sep">·</span>
            <span class="ann-page">p.&nbsp;{{ a.page }}</span>
          </div>
          <template v-if="editingId === a.id">
            <textarea
              v-model="editDraft"
              class="note-edit"
              rows="2"
              placeholder="Add a note…"
              @keydown.enter.ctrl="commitEdit(a.id)"
              @keydown.escape="cancelEdit"
            ></textarea>
            <div class="note-edit-actions">
              <button class="note-save" @click="commitEdit(a.id)">Save</button>
              <button class="note-cancel" @click="cancelEdit">Cancel</button>
            </div>
          </template>
          <div v-else-if="a.noteText" class="ann-note" @click="startEdit(a)">{{ a.noteText }}</div>
          <button v-else class="note-add-btn" @click="startEdit(a)">+ note</button>
        </div>
        <div class="ann-actions">
          <button class="action-btn" title="Open in PDF" @click="openInPdf(a)">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
              <path d="M2 2h8v8"/><path d="M2 10L10 2"/>
            </svg>
          </button>
          <button class="action-btn action-del" title="Delete annotation" @click="deleteAndRefresh(a.id)">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
              <path d="M2 3h8M5 3V2h2v1M4 3v6.5a.5.5 0 00.5.5h3a.5.5 0 00.5-.5V3"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ann-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: var(--bg);
}

/* ── Toolbar ─────────────────────────────────────────────────────────────── */

.ann-toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 14px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-chrome);
  flex-shrink: 0;
}

.search-wrap {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 7px;
  background: var(--surface-solid);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius);
  padding: 0 9px;
  height: 28px;
}

.search-icon {
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  font-size: 12.5px;
  color: var(--text);
  font-family: var(--font-ui);
}

.search-input::placeholder { color: var(--text-tertiary); }

.search-clear {
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 0;
  display: flex;
  align-items: center;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.ann-count {
  font-size: 11px;
  color: var(--text-tertiary);
  font-variant-numeric: tabular-nums;
}

.group-toggle {
  background: none;
  border: 1px solid transparent;
  width: 26px;
  height: 26px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
  cursor: pointer;
  transition: background var(--t), border-color var(--t), color var(--t);
}

.group-toggle:hover { background: var(--bg-chrome-active); color: var(--text-secondary); }
.group-toggle.active { background: var(--accent-soft); border-color: rgba(10,95,191,0.2); color: var(--accent); }

/* ── Empty state ─────────────────────────────────────────────────────────── */

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--text-tertiary);
  padding: 40px;
  text-align: center;
}

.empty-label { font-size: 12.5px; max-width: 280px; line-height: 1.5; }

/* ── List ────────────────────────────────────────────────────────────────── */

.ann-list {
  flex: 1;
  overflow-y: auto;
  padding: 6px 0;
}

/* ── Group header ────────────────────────────────────────────────────────── */

.group-header {
  display: flex;
  align-items: baseline;
  gap: 7px;
  padding: 10px 16px 5px;
  position: sticky;
  top: 0;
  background: var(--bg);
  z-index: 1;
}

.group:not(:first-child) .group-header { border-top: 1px solid var(--border); }

.group-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  min-width: 0;
}

.group-sub {
  font-size: 11px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.group-count {
  font-size: 10px;
  font-weight: 600;
  color: var(--text-tertiary);
  background: var(--bg-chrome-active);
  border-radius: 9px;
  padding: 1px 6px;
  flex-shrink: 0;
}

/* ── Row ─────────────────────────────────────────────────────────────────── */

.ann-row {
  display: flex;
  align-items: flex-start;
  gap: 0;
  padding: 7px 14px 7px 0;
  border-bottom: 1px solid var(--border);
  transition: background var(--t);
}

.ann-row:last-child { border-bottom: none; }
.ann-row:hover { background: var(--bg-chrome); }

.color-stripe {
  width: 3px;
  min-height: 100%;
  align-self: stretch;
  flex-shrink: 0;
  margin-right: 11px;
  border-radius: 0 2px 2px 0;
  opacity: 0.7;
}

.ann-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.ann-text { cursor: pointer; }

.ann-quote {
  font-size: 12.5px;
  color: var(--text);
  font-family: var(--font-doc);
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.ann-quote--empty { color: var(--text-tertiary); font-style: italic; }

.ann-meta {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 10.5px;
  color: var(--text-tertiary);
}

.ann-source { font-weight: 500; color: var(--text-secondary); }
.meta-sep { color: var(--border-medium); }
.ann-page { font-variant-numeric: tabular-nums; }
.ann-file { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 180px; }

.ann-note {
  font-size: 11.5px;
  color: var(--text-secondary);
  font-style: italic;
  line-height: 1.45;
  cursor: pointer;
  padding: 2px 0;
}

.ann-note:hover { color: var(--text); }

.note-add-btn {
  background: none;
  border: none;
  font-size: 11px;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 2px 0;
  text-align: left;
  opacity: 0;
  transition: opacity var(--t), color var(--t);
}

.ann-row:hover .note-add-btn { opacity: 1; }
.note-add-btn:hover { color: var(--accent); }

.note-edit {
  width: 100%;
  font-size: 11.5px;
  font-family: var(--font-ui);
  color: var(--text);
  background: var(--surface-solid);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  padding: 5px 8px;
  resize: none;
  outline: none;
  line-height: 1.45;
}

.note-edit:focus { border-color: var(--accent); }

.note-edit-actions {
  display: flex;
  gap: 6px;
}

.note-save, .note-cancel {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-medium);
  cursor: pointer;
  font-family: var(--font-ui);
  transition: background var(--t), border-color var(--t), color var(--t);
}

.note-save {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}

.note-save:hover { opacity: 0.9; }

.note-cancel {
  background: none;
  color: var(--text-secondary);
}

.note-cancel:hover { background: var(--bg-chrome-active); }

/* ── Row actions ─────────────────────────────────────────────────────────── */

.ann-actions {
  display: flex;
  flex-direction: column;
  gap: 3px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity var(--t);
  padding-top: 1px;
}

.ann-row:hover .ann-actions { opacity: 1; }

.action-btn {
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
  transition: background var(--t), border-color var(--t), color var(--t);
}

.action-btn:hover { background: var(--bg-chrome-active); color: var(--text-secondary); }
.action-del:hover { background: #fff1f0; border-color: rgba(200,50,50,0.2); color: #c83232; }
</style>
