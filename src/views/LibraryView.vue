<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useLibrary, type LibraryItem } from '../composables/useLibrary'
import LibraryEntryForm from '../components/LibraryEntryForm.vue'

const { items, loading, loadItems, deleteItem, createItem } = useLibrary()

onMounted(loadItems)

// ── Sorting ───────────────────────────────────────────────────────────────────

type SortKey = 'title' | 'authors' | 'year' | 'entryType'
const sortKey = ref<SortKey>('title')
const sortAsc = ref(true)

function setSort(key: SortKey) {
  if (sortKey.value === key) {
    sortAsc.value = !sortAsc.value
  } else {
    sortKey.value = key
    sortAsc.value = true
  }
}

const sorted = computed(() => {
  const list = [...items.value]
  list.sort((a, b) => {
    const av = (a[sortKey.value] ?? '').toLowerCase()
    const bv = (b[sortKey.value] ?? '').toLowerCase()
    return sortAsc.value ? av.localeCompare(bv) : bv.localeCompare(av)
  })
  return list
})

// ── Detail panel ──────────────────────────────────────────────────────────────

const activeItem = ref<LibraryItem | null>(null)
const panelOpen = ref(false)

function openItem(item: LibraryItem) {
  activeItem.value = item
  panelOpen.value = true
}

function closePanel() {
  panelOpen.value = false
  activeItem.value = null
}

// ── Form (add / edit) ─────────────────────────────────────────────────────────

const formOpen = ref(false)
const formEditItem = ref<LibraryItem | null>(null)

function openAddForm() {
  formEditItem.value = null
  formOpen.value = true
}

function openEditForm(item: LibraryItem) {
  formEditItem.value = item
  formOpen.value = true
}

function onFormSaved(saved: LibraryItem) {
  formOpen.value = false
  // If we were editing the active item, refresh it in the panel
  if (activeItem.value?.id === saved.id) {
    activeItem.value = saved
  }
}

async function duplicateItem(item: LibraryItem) {
  const { id: _id, addedAt: _a, updatedAt: _u, tags: _t, ...fields } = item
  await createItem({ ...fields, key: `${item.key}_copy` })
}

// ── Delete ────────────────────────────────────────────────────────────────────

const confirmingDelete = ref<LibraryItem | null>(null)

function requestDelete(item: LibraryItem) {
  confirmingDelete.value = item
}

async function confirmDelete() {
  if (!confirmingDelete.value) return
  const id = confirmingDelete.value.id
  await deleteItem(id)
  if (activeItem.value?.id === id) closePanel()
  confirmingDelete.value = null
}

// ── Helpers ───────────────────────────────────────────────────────────────────

const TYPE_COLORS: Record<string, string> = {
  article:      '#0A5FBF',
  techreport:   '#E8650A',
  book:         '#7C3AED',
  inproceedings:'#16963F',
  misc:         '#A5A4A2',
}

function typeColor(entryType: string): string {
  return TYPE_COLORS[entryType] ?? TYPE_COLORS.misc
}

function typeLabel(entryType: string): string {
  const map: Record<string, string> = {
    article:       'art',
    techreport:    'rep',
    book:          'bk',
    inproceedings: 'conf',
    misc:          'misc',
  }
  return map[entryType] ?? entryType.slice(0, 4)
}

function venue(item: LibraryItem): string {
  return item.journal ?? item.booktitle ?? item.publisher ?? item.institution ?? ''
}
</script>

<template>
  <div class="lib-layout">

    <!-- ── Main list ─────────────────────────────────────────────────────────── -->
    <div class="lib-main">
      <div class="lib-header">
        <h2 class="lib-title">Library</h2>
        <span class="lib-count" v-if="!loading">{{ items.length }} item{{ items.length !== 1 ? 's' : '' }}</span>
        <span class="lib-count loading" v-else>Loading…</span>
        <button class="add-btn" @click="openAddForm" title="Add entry">
          <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <line x1="6.5" y1="1" x2="6.5" y2="12"/>
            <line x1="1" y1="6.5" x2="12" y2="6.5"/>
          </svg>
          Add
        </button>
      </div>

      <div class="lib-table-wrap">
        <table class="lib-table" v-if="sorted.length > 0">
          <thead>
            <tr>
              <th class="col-type">Type</th>
              <th
                class="col-title sortable"
                :class="{ sorted: sortKey === 'title' }"
                @click="setSort('title')"
              >
                Title
                <span class="sort-icon">{{ sortKey === 'title' ? (sortAsc ? '↑' : '↓') : '' }}</span>
              </th>
              <th
                class="col-authors sortable"
                :class="{ sorted: sortKey === 'authors' }"
                @click="setSort('authors')"
              >
                Authors
                <span class="sort-icon">{{ sortKey === 'authors' ? (sortAsc ? '↑' : '↓') : '' }}</span>
              </th>
              <th
                class="col-year sortable"
                :class="{ sorted: sortKey === 'year' }"
                @click="setSort('year')"
              >
                Year
                <span class="sort-icon">{{ sortKey === 'year' ? (sortAsc ? '↑' : '↓') : '' }}</span>
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in sorted"
              :key="item.id"
              class="lib-row"
              :class="{ active: activeItem?.id === item.id }"
              @click="openItem(item)"
            >
              <td class="col-type">
                <span
                  class="type-badge"
                  :style="{ background: typeColor(item.entryType) + '18', color: typeColor(item.entryType) }"
                >{{ typeLabel(item.entryType) }}</span>
              </td>
              <td class="col-title">
                <span class="row-title">{{ item.title ?? item.key }}</span>
              </td>
              <td class="col-authors">{{ item.authors ?? '—' }}</td>
              <td class="col-year">{{ item.year ?? '—' }}</td>
            </tr>
          </tbody>
        </table>

        <div class="lib-empty" v-else-if="!loading">
          <p>No items in library.</p>
          <p class="lib-empty-sub">Add references via the Library menu, or import a .bib file.</p>
        </div>
      </div>
    </div>

    <!-- ── Detail panel ──────────────────────────────────────────────────────── -->
    <Transition name="panel">
      <div class="detail-panel" v-if="panelOpen && activeItem">
        <div class="dp-header">
          <span class="dp-label">Source Detail</span>
          <button class="dp-close" @click="closePanel" title="Close">
            <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round">
              <line x1="1" y1="1" x2="12" y2="12"/>
              <line x1="12" y1="1" x2="1" y2="12"/>
            </svg>
          </button>
        </div>

        <div class="dp-body">
          <div class="dp-type-row">
            <span
              class="type-badge"
              :style="{ background: typeColor(activeItem.entryType) + '18', color: typeColor(activeItem.entryType) }"
            >{{ typeLabel(activeItem.entryType) }}</span>
            <span class="dp-key">{{ activeItem.key }}</span>
          </div>

          <p class="dp-title">{{ activeItem.title }}</p>

          <div class="dp-meta">
            <span class="dp-authors" v-if="activeItem.authors">{{ activeItem.authors }}</span>
            <span class="dp-venue" v-if="venue(activeItem) || activeItem.year">
              {{ venue(activeItem) }}<template v-if="venue(activeItem) && activeItem.year"> · </template>{{ activeItem.year }}
            </span>
            <a
              v-if="activeItem.doi"
              class="dp-doi"
              :href="`https://doi.org/${activeItem.doi}`"
              target="_blank"
              rel="noopener"
            >{{ activeItem.doi }}</a>
          </div>

          <div class="dp-fields" v-if="activeItem.volume || activeItem.pages || activeItem.number || activeItem.isbn">
            <div class="dp-field" v-if="activeItem.volume">
              <span class="dp-field-label">Vol</span>
              <span>{{ activeItem.volume }}</span>
            </div>
            <div class="dp-field" v-if="activeItem.number">
              <span class="dp-field-label">No.</span>
              <span>{{ activeItem.number }}</span>
            </div>
            <div class="dp-field" v-if="activeItem.pages">
              <span class="dp-field-label">Pp.</span>
              <span>{{ activeItem.pages }}</span>
            </div>
            <div class="dp-field" v-if="activeItem.isbn">
              <span class="dp-field-label">ISBN</span>
              <span>{{ activeItem.isbn }}</span>
            </div>
          </div>

          <div class="dp-block" v-if="activeItem.abstractText">
            <div class="dp-block-label">Abstract</div>
            <p class="dp-abstract">{{ activeItem.abstractText }}</p>
          </div>

          <div class="dp-block" v-if="activeItem.keywords">
            <div class="dp-block-label">Keywords</div>
            <p class="dp-keywords">{{ activeItem.keywords }}</p>
          </div>
        </div>

        <div class="dp-actions">
          <button class="dp-action-btn" @click="openEditForm(activeItem)" title="Edit entry">
            <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M9 1.5L11.5 4L5 10.5H2.5V8L9 1.5Z"/>
            </svg>
            Edit
          </button>
          <button class="dp-action-btn" @click="duplicateItem(activeItem)" title="Duplicate entry">
            <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <rect x="4.5" y="4.5" width="7" height="7" rx="1"/>
              <path d="M4.5 8.5H2.5a1 1 0 01-1-1v-5a1 1 0 011-1h5a1 1 0 011 1v2"/>
            </svg>
            Duplicate
          </button>
          <button class="dp-action-btn danger" @click="requestDelete(activeItem)" title="Delete entry">
            <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M2 3.5h9M5 3.5V2h3v1.5M5.5 5.5v4M7.5 5.5v4M3 3.5l.5 7h6l.5-7"/>
            </svg>
            Delete
          </button>
        </div>
      </div>
    </Transition>
  </div>

  <!-- Entry form modal -->
  <LibraryEntryForm
    v-if="formOpen"
    :edit-item="formEditItem"
    @close="formOpen = false"
    @saved="onFormSaved"
  />

  <!-- Delete confirmation -->
  <Teleport to="body">
    <div class="confirm-backdrop" v-if="confirmingDelete" @click.self="confirmingDelete = null">
      <div class="confirm-dialog">
        <p class="confirm-msg">Delete <strong>{{ confirmingDelete.key }}</strong>?</p>
        <p class="confirm-sub">This will remove the entry from your library and cannot be undone.</p>
        <div class="confirm-actions">
          <button class="fs-btn secondary" @click="confirmingDelete = null">Cancel</button>
          <button class="fs-btn danger" @click="confirmDelete">Delete</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.lib-layout {
  display: flex;
  height: 100%;
  overflow: hidden;
  background: var(--bg);
}

/* ── Main list ─────────────────────────────────────────────────────────────── */

.lib-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.lib-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 20px 12px 24px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.add-btn {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 5px;
  height: 28px;
  padding: 0 12px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: opacity var(--t);
}
.add-btn:hover { opacity: 0.88; }

.lib-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text);
  letter-spacing: -0.01em;
}

.lib-count {
  font-size: 12px;
  color: var(--text-tertiary);
  font-variant-numeric: tabular-nums;
}

.lib-count.loading {
  font-style: italic;
}

.lib-table-wrap {
  flex: 1;
  overflow-y: auto;
}

.lib-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.lib-table thead tr {
  border-bottom: 1px solid var(--border);
  background: var(--bg-chrome);
  position: sticky;
  top: 0;
  z-index: 1;
}

.lib-table th {
  padding: 8px 12px 8px;
  font-size: 10.5px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--text-tertiary);
  text-align: left;
  white-space: nowrap;
  user-select: none;
}

.lib-table th.sortable {
  cursor: pointer;
}

.lib-table th.sortable:hover {
  color: var(--text-secondary);
}

.lib-table th.sorted {
  color: var(--accent);
}

.sort-icon {
  margin-left: 3px;
  font-size: 10px;
}

.lib-row {
  border-bottom: 1px solid var(--border);
  cursor: pointer;
  transition: background var(--t);
}

.lib-row:hover {
  background: var(--bg-chrome);
}

.lib-row.active {
  background: var(--accent-soft);
}

.lib-table td {
  padding: 10px 12px;
  vertical-align: middle;
}

.col-type {
  width: 52px;
}

.col-authors {
  width: 200px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 200px;
}

.col-year {
  width: 60px;
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
  text-align: right;
  padding-right: 20px;
}

.row-title {
  font-weight: 500;
  color: var(--text);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  line-height: 1.4;
}

.type-badge {
  display: inline-block;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  padding: 2px 6px;
  border-radius: var(--radius-xs);
  white-space: nowrap;
}

.lib-empty {
  padding: 60px 24px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 14px;
}

.lib-empty-sub {
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-tertiary);
}

/* ── Detail panel ──────────────────────────────────────────────────────────── */

.detail-panel {
  width: 284px;
  flex-shrink: 0;
  border-left: 1px solid var(--border);
  background: var(--surface-solid);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.dp-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px 10px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.dp-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  color: var(--text-tertiary);
}

.dp-close {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-tertiary);
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  transition: background var(--t), color var(--t);
}
.dp-close:hover {
  background: var(--bg-chrome-active);
  color: var(--text);
}

.dp-body {
  padding: 16px 16px 20px;
  overflow-y: auto;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.dp-type-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.dp-key {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-tertiary);
}

.dp-title {
  font-family: var(--font-doc);
  font-size: 14px;
  font-weight: 700;
  line-height: 1.45;
  color: var(--text);
}

.dp-meta {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.dp-authors {
  font-size: 12px;
  font-weight: 500;
  color: var(--text);
}

.dp-venue {
  font-size: 11px;
  color: var(--text-secondary);
}

.dp-doi {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--accent);
  text-decoration: none;
  word-break: break-all;
}
.dp-doi:hover { text-decoration: underline; }

.dp-fields {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 14px;
}

.dp-field {
  display: flex;
  align-items: baseline;
  gap: 4px;
  font-size: 11.5px;
}

.dp-field-label {
  font-size: 9.5px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--text-tertiary);
}

.dp-block {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.dp-block-label {
  font-size: 9.5px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  color: var(--text-tertiary);
}

.dp-abstract {
  font-family: var(--font-doc);
  font-size: 12.5px;
  line-height: 1.65;
  color: var(--text-secondary);
}

.dp-keywords {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
}

/* ── Detail panel actions ──────────────────────────────────────────────────── */

.dp-actions {
  display: flex;
  flex-direction: column;
  gap: 1px;
  padding: 8px 10px 10px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.dp-action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 30px;
  padding: 0 10px;
  border: none;
  border-radius: var(--radius-sm);
  background: none;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background var(--t), color var(--t);
  text-align: left;
}

.dp-action-btn:hover {
  background: var(--bg-chrome-active);
  color: var(--text);
}

.dp-action-btn.danger { color: #C0392B; }
.dp-action-btn.danger:hover { background: rgba(192, 57, 43, 0.08); }

/* ── Confirm dialog ────────────────────────────────────────────────────────── */

.confirm-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  backdrop-filter: blur(3px);
  -webkit-backdrop-filter: blur(3px);
  z-index: 1100;
  display: flex;
  align-items: center;
  justify-content: center;
}

.confirm-dialog {
  background: var(--surface-solid);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  padding: 22px 24px 18px;
  max-width: 340px;
  width: 100%;
}

.confirm-msg {
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
  margin-bottom: 6px;
}

.confirm-sub {
  font-size: 12.5px;
  color: var(--text-secondary);
  line-height: 1.5;
  margin-bottom: 18px;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.fs-btn {
  height: 32px;
  padding: 0 16px;
  border-radius: var(--radius);
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  font-family: var(--font-ui);
  transition: opacity var(--t), background var(--t);
}

.fs-btn.secondary {
  background: var(--bg-chrome-active);
  color: var(--text-secondary);
}
.fs-btn.secondary:hover { opacity: 0.8; }

.fs-btn.danger {
  background: #C0392B;
  color: #fff;
}
.fs-btn.danger:hover { opacity: 0.88; }

/* ── Transition ────────────────────────────────────────────────────────────── */

.panel-enter-active,
.panel-leave-active {
  transition: transform var(--t), opacity var(--t);
}
.panel-enter-from,
.panel-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>
