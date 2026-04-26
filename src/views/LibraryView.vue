<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useLibrary, type LibraryItem, type Attachment, type ImportResult } from '../composables/useLibrary'
import LibraryEntryForm from '../components/LibraryEntryForm.vue'
import CollectionsSidebar from '../components/CollectionsSidebar.vue'

const router = useRouter()

const {
  items, displayItems, loading, collections, tags, filterQuery,
  loadItems, loadCollections, loadTags, applyFilter,
  deleteItem, createItem,
  addItemToCollection, removeItemFromCollection, getItemCollectionIds,
  importBibFile, exportBibFile,
  pickAndAttachFile, getItemAttachments, removeAttachment, openAttachmentExternal,
} = useLibrary()

onMounted(() => {
  loadItems()
  loadCollections()
  loadTags()
})

// ── Tag color lookup ──────────────────────────────────────────────────────────

const tagColorMap = computed(() => {
  const map = new Map<string, string>()
  for (const t of tags.value) map.set(t.name, t.color)
  return map
})

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
  const list = [...displayItems.value]
  list.sort((a, b) => {
    const av = (a[sortKey.value] ?? '').toLowerCase()
    const bv = (b[sortKey.value] ?? '').toLowerCase()
    return sortAsc.value ? av.localeCompare(bv) : bv.localeCompare(av)
  })
  return list
})

// ── Search + filter (F6) ──────────────────────────────────────────────────────

const searchText = ref('')
const yearMin = ref('')
const yearMax = ref('')

const TYPE_OPTIONS = ['article', 'book', 'inproceedings', 'techreport', 'misc'] as const
const selectedTypes = ref<string[]>([])

let searchTimer: ReturnType<typeof setTimeout> | null = null

function scheduleFilter() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    filterQuery.value = {
      ...filterQuery.value,
      text: searchText.value || undefined,
      entryTypes: selectedTypes.value.length ? selectedTypes.value : undefined,
      yearMin: yearMin.value || undefined,
      yearMax: yearMax.value || undefined,
    }
    applyFilter()
  }, 200)
}

function toggleType(t: string) {
  const idx = selectedTypes.value.indexOf(t)
  if (idx === -1) selectedTypes.value = [...selectedTypes.value, t]
  else selectedTypes.value = selectedTypes.value.filter(x => x !== t)
  scheduleFilter()
}

function toggleDoi() {
  filterQuery.value = { ...filterQuery.value, hasDoi: filterQuery.value.hasDoi ? undefined : true }
  applyFilter()
}

const hasActiveFilter = computed(() =>
  !!searchText.value || selectedTypes.value.length > 0 ||
  !!yearMin.value || !!yearMax.value || !!filterQuery.value.hasDoi
)

function clearFilters() {
  searchText.value = ''
  yearMin.value = ''
  yearMax.value = ''
  selectedTypes.value = []
  filterQuery.value = {
    collectionId: filterQuery.value.collectionId,
    tagName: filterQuery.value.tagName,
  }
  applyFilter()
}

// ── Import ────────────────────────────────────────────────────────────────────

const importOpen       = ref(false)
const importFilePath   = ref('')
const importMode       = ref<'merge' | 'replace'>('merge')
const importing        = ref(false)
const importResult     = ref<ImportResult | null>(null)
const importError      = ref('')

async function pickImportFile() {
  const p = await invoke<string | null>('pick_import_bib')
  if (p) {
    importFilePath.value = p
    importResult.value = null
    importError.value = ''
  }
}

async function doImport() {
  if (!importFilePath.value) return
  importing.value = true
  importError.value = ''
  try {
    importResult.value = await importBibFile(importFilePath.value, importMode.value)
  } catch (e) {
    importError.value = String(e)
  } finally {
    importing.value = false
  }
}

function closeImport() {
  importOpen.value  = false
  importFilePath.value = ''
  importResult.value = null
  importError.value = ''
}

// ── Export ────────────────────────────────────────────────────────────────────

const exportingAll = ref(false)

async function doExport() {
  const p = await invoke<string | null>('pick_export_bib')
  if (!p) return
  exportingAll.value = true
  try {
    const ids = hasActiveFilter.value || filterQuery.value.collectionId || filterQuery.value.tagName
      ? displayItems.value.map(i => i.id)
      : []
    await exportBibFile(ids, p)
  } finally {
    exportingAll.value = false
  }
}

// ── Detail panel ──────────────────────────────────────────────────────────────

const activeItem = ref<LibraryItem | null>(null)
const panelOpen = ref(false)
const activeItemCollectionIds = ref<number[]>([])
const addToCollId = ref<number | ''>('')

// ── Attachments ───────────────────────────────────────────────────────────────

const attachments = ref<Attachment[]>([])
const attachingFile = ref(false)

async function loadAttachments(itemId: number) {
  attachments.value = await getItemAttachments(itemId)
}

async function attachFile() {
  if (!activeItem.value) return
  attachingFile.value = true
  try {
    const att = await pickAndAttachFile(activeItem.value.id)
    if (att) attachments.value.push(att)
  } finally {
    attachingFile.value = false
  }
}

async function deleteAttachment(id: number) {
  await removeAttachment(id)
  attachments.value = attachments.value.filter(a => a.id !== id)
}

function openInViewer(att: Attachment) {
  router.push({
    path: '/pdf',
    query: {
      id:     att.id.toString(),
      itemId: (activeItem.value?.id ?? 0).toString(),
      name:   att.fileName,
    },
  })
}

async function openExternal(id: number) {
  await openAttachmentExternal(id)
}

// ── Detail panel ──────────────────────────────────────────────────────────────

async function openItem(item: LibraryItem) {
  activeItem.value = item
  panelOpen.value = true
  activeItemCollectionIds.value = await getItemCollectionIds(item.id)
  await loadAttachments(item.id)
}

function closePanel() {
  panelOpen.value = false
  activeItem.value = null
  activeItemCollectionIds.value = []
  attachments.value = []
}

watch(activeItem, async (item) => {
  if (!item) { activeItemCollectionIds.value = []; return }
  activeItemCollectionIds.value = await getItemCollectionIds(item.id)
})

const activeItemCollections = computed(() =>
  collections.value.filter(c => activeItemCollectionIds.value.includes(c.id))
)

const collectionsForAdd = computed(() =>
  collections.value.filter(c => !activeItemCollectionIds.value.includes(c.id))
)

async function addToCol() {
  if (!activeItem.value || !addToCollId.value) return
  await addItemToCollection(Number(addToCollId.value), activeItem.value.id)
  activeItemCollectionIds.value = await getItemCollectionIds(activeItem.value.id)
  addToCollId.value = ''
}

async function removeFromCol(collId: number) {
  if (!activeItem.value) return
  await removeItemFromCollection(collId, activeItem.value.id)
  activeItemCollectionIds.value = await getItemCollectionIds(activeItem.value.id)
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
  article:       '#0A5FBF',
  techreport:    '#E8650A',
  book:          '#7C3AED',
  inproceedings: '#16963F',
  misc:          '#A5A4A2',
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

const countLabel = computed(() => {
  const d = displayItems.value.length
  const t = items.value.length
  if (d === t) return `${t} item${t !== 1 ? 's' : ''}`
  return `${d} of ${t}`
})
</script>

<template>
  <div class="lib-layout">

    <!-- ── Left sidebar ─────────────────────────────────────────────────────── -->
    <CollectionsSidebar />

    <!-- ── Main list ─────────────────────────────────────────────────────────── -->
    <div class="lib-main">
      <div class="lib-header">
        <div class="lib-header-top">
          <h2 class="lib-title">Library</h2>
          <span class="lib-count" :class="{ loading }">
            {{ loading ? 'Loading…' : countLabel }}
          </span>
          <div class="lib-search-wrap">
            <svg class="search-icon" width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
              <circle cx="5" cy="5" r="3.5"/>
              <line x1="7.8" y1="7.8" x2="11" y2="11"/>
            </svg>
            <input
              v-model="searchText"
              class="lib-search"
              placeholder="Search entries…"
              @input="scheduleFilter"
            />
            <button v-if="searchText" class="search-clear" @click="searchText = ''; scheduleFilter()">
              <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
                <line x1="1" y1="1" x2="8" y2="8"/>
                <line x1="8" y1="1" x2="1" y2="8"/>
              </svg>
            </button>
          </div>
          <button class="toolbar-btn" @click="importOpen = true" title="Import .bib file">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M6 1v7M3 5l3 3 3-3"/>
              <path d="M2 10h8"/>
            </svg>
            Import
          </button>
          <button class="toolbar-btn" @click="doExport" :disabled="exportingAll" title="Export .bib file">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M6 8V1M3 4l3-3 3 3"/>
              <path d="M2 10h8"/>
            </svg>
            Export
          </button>
          <button class="add-btn" @click="openAddForm" title="Add entry">
            <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <line x1="5.5" y1="1" x2="5.5" y2="10"/>
              <line x1="1" y1="5.5" x2="10" y2="5.5"/>
            </svg>
            Add
          </button>
        </div>

        <div class="lib-filter-bar">
          <button
            v-for="t in TYPE_OPTIONS"
            :key="t"
            class="filter-pill"
            :class="{ active: selectedTypes.includes(t) }"
            :style="selectedTypes.includes(t) ? { background: typeColor(t) + '18', color: typeColor(t), borderColor: typeColor(t) + '60' } : {}"
            @click="toggleType(t)"
          >{{ typeLabel(t) }}</button>

          <span class="filter-sep"></span>

          <div class="year-range">
            <input
              v-model="yearMin"
              class="year-input"
              placeholder="From"
              maxlength="4"
              @input="scheduleFilter"
            />
            <span class="year-dash">–</span>
            <input
              v-model="yearMax"
              class="year-input"
              placeholder="To"
              maxlength="4"
              @input="scheduleFilter"
            />
          </div>

          <button
            class="filter-pill"
            :class="{ active: !!filterQuery.hasDoi }"
            :style="filterQuery.hasDoi ? { background: '#16963F18', color: '#16963F', borderColor: '#16963F60' } : {}"
            @click="toggleDoi"
          >Has DOI</button>

          <button v-if="hasActiveFilter" class="filter-clear-btn" @click="clearFilters">
            Clear filters
          </button>
        </div>
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
              >Title <span class="sort-icon">{{ sortKey === 'title' ? (sortAsc ? '↑' : '↓') : '' }}</span></th>
              <th
                class="col-authors sortable"
                :class="{ sorted: sortKey === 'authors' }"
                @click="setSort('authors')"
              >Authors <span class="sort-icon">{{ sortKey === 'authors' ? (sortAsc ? '↑' : '↓') : '' }}</span></th>
              <th
                class="col-year sortable"
                :class="{ sorted: sortKey === 'year' }"
                @click="setSort('year')"
              >Year <span class="sort-icon">{{ sortKey === 'year' ? (sortAsc ? '↑' : '↓') : '' }}</span></th>
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
                <div class="row-tags" v-if="item.tags && item.tags.length > 0">
                  <span
                    v-for="tagName in item.tags"
                    :key="tagName"
                    class="row-tag-chip"
                    :style="{
                      background: (tagColorMap.get(tagName) ?? '#A5A4A2') + '22',
                      color: tagColorMap.get(tagName) ?? '#A5A4A2',
                      borderColor: (tagColorMap.get(tagName) ?? '#A5A4A2') + '55',
                    }"
                  >{{ tagName }}</span>
                </div>
              </td>
              <td class="col-authors">{{ item.authors ?? '—' }}</td>
              <td class="col-year">{{ item.year ?? '—' }}</td>
            </tr>
          </tbody>
        </table>

        <div class="lib-empty" v-else-if="!loading">
          <p>{{ hasActiveFilter || filterQuery.collectionId || filterQuery.tagName ? 'No items match the current filter.' : 'No items in library.' }}</p>
          <p class="lib-empty-sub" v-if="!hasActiveFilter && !filterQuery.collectionId && !filterQuery.tagName">
            Add references via the Add button, or import a .bib file.
          </p>
        </div>
      </div>
    </div>

    <!-- ── Detail panel ──────────────────────────────────────────────────────── -->
    <Transition name="panel">
      <div class="detail-panel" v-if="panelOpen && activeItem">
        <div class="dp-header">
          <span class="dp-label">Source Detail</span>
          <button class="dp-close" @click="closePanel" title="Close">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round">
              <line x1="1" y1="1" x2="11" y2="11"/>
              <line x1="11" y1="1" x2="1" y2="11"/>
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

          <!-- Tags -->
          <div class="dp-block" v-if="activeItem.tags && activeItem.tags.length > 0">
            <div class="dp-block-label">Tags</div>
            <div class="dp-tags">
              <span
                v-for="tagName in activeItem.tags"
                :key="tagName"
                class="dp-tag-chip"
                :style="{
                  background: (tagColorMap.get(tagName) ?? '#A5A4A2') + '22',
                  color: tagColorMap.get(tagName) ?? '#A5A4A2',
                  borderColor: (tagColorMap.get(tagName) ?? '#A5A4A2') + '55',
                }"
              >{{ tagName }}</span>
            </div>
          </div>

          <!-- Collections membership -->
          <div class="dp-block" v-if="collections.length > 0">
            <div class="dp-block-label">Collections</div>
            <div class="dp-coll-chips" v-if="activeItemCollections.length > 0">
              <span
                v-for="col in activeItemCollections"
                :key="col.id"
                class="dp-coll-chip"
              >
                {{ col.name }}
                <button class="dp-coll-remove" @click="removeFromCol(col.id)" title="Remove from collection">×</button>
              </span>
            </div>
            <div v-if="collectionsForAdd.length > 0" class="dp-coll-add">
              <select v-model="addToCollId" class="dp-coll-select">
                <option value="">+ Add to collection…</option>
                <option v-for="col in collectionsForAdd" :key="col.id" :value="col.id">{{ col.name }}</option>
              </select>
              <button v-if="addToCollId" class="dp-coll-add-btn" @click="addToCol">Add</button>
            </div>
            <div v-if="activeItemCollections.length === 0 && collectionsForAdd.length === 0" class="dp-empty-hint">
              No collections
            </div>
          </div>

          <!-- Files (attachments) -->
          <div class="dp-block">
            <div class="dp-block-label dp-files-header">
              Files
              <button class="dp-attach-btn" @click="attachFile" :disabled="attachingFile" title="Attach PDF">
                <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
                  <line x1="5.5" y1="1" x2="5.5" y2="10"/>
                  <line x1="1" y1="5.5" x2="10" y2="5.5"/>
                </svg>
              </button>
            </div>
            <div v-if="attachments.length === 0" class="dp-empty-hint">No files attached</div>
            <div v-for="att in attachments" :key="att.id" class="dp-file-row">
              <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="dp-file-icon">
                <rect x="1.5" y="0.5" width="8" height="10" rx="1"/>
                <line x1="3.5" y1="4" x2="7.5" y2="4"/>
                <line x1="3.5" y1="6" x2="7.5" y2="6"/>
                <line x1="3.5" y1="8" x2="5.5" y2="8"/>
              </svg>
              <span class="dp-file-name" @click="openInViewer(att)" title="Open in Quire viewer">{{ att.fileName }}</span>
              <button class="dp-file-ext" @click="openExternal(att.id)" title="Open in system PDF reader">↗</button>
              <button class="dp-file-del" @click="deleteAttachment(att.id)" title="Remove attachment">×</button>
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
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M8.5 1.5L10.5 3.5L4.5 9.5H2.5V7.5L8.5 1.5Z"/>
            </svg>
            Edit
          </button>
          <button class="dp-action-btn" @click="duplicateItem(activeItem)" title="Duplicate entry">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <rect x="4" y="4" width="7" height="7" rx="1"/>
              <path d="M4 8H2.5a1 1 0 01-1-1V2.5a1 1 0 011-1H7a1 1 0 011 1V4"/>
            </svg>
            Duplicate
          </button>
          <button class="dp-action-btn danger" @click="requestDelete(activeItem)" title="Delete entry">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M2 3.5h8M4.5 3.5V2h3v1.5M5 5.5v3.5M7 5.5v3.5M2.5 3.5l.5 6.5h6l.5-6.5"/>
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

  <!-- Import modal -->
  <Teleport to="body">
    <div class="confirm-backdrop" v-if="importOpen" @click.self="closeImport">
      <div class="confirm-dialog import-dialog">
        <p class="confirm-msg">Import .bib file</p>

        <div class="import-file-row">
          <span class="import-path">{{ importFilePath || 'No file selected' }}</span>
          <button class="fs-btn secondary" @click="pickImportFile">Browse…</button>
        </div>

        <div class="import-mode">
          <label class="mode-label">
            <input type="radio" v-model="importMode" value="merge" />
            <span>Merge — skip entries with duplicate keys</span>
          </label>
          <label class="mode-label">
            <input type="radio" v-model="importMode" value="replace" />
            <span>Replace — overwrite existing entries by key</span>
          </label>
        </div>

        <div class="import-result" v-if="importResult">
          <span class="result-added">{{ importResult.added }} added</span>
          <span class="result-skipped" v-if="importResult.skipped > 0">· {{ importResult.skipped }} skipped</span>
          <span class="result-errors"  v-if="importResult.errors.length > 0">· {{ importResult.errors.length }} errors</span>
        </div>

        <p class="fetch-error" v-if="importError" style="margin:8px 0 0">{{ importError }}</p>

        <div class="confirm-actions" style="margin-top:16px">
          <button class="fs-btn secondary" @click="closeImport">
            {{ importResult ? 'Done' : 'Cancel' }}
          </button>
          <button
            class="fs-btn primary"
            :disabled="!importFilePath || importing"
            @click="doImport"
            v-if="!importResult"
          >{{ importing ? 'Importing…' : 'Import' }}</button>
        </div>
      </div>
    </div>
  </Teleport>

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
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.lib-header-top {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px 10px 20px;
}

.lib-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
  letter-spacing: -0.01em;
  flex-shrink: 0;
}

.lib-count {
  font-size: 11.5px;
  color: var(--text-tertiary);
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}

.lib-count.loading {
  font-style: italic;
}

/* Search */
.lib-search-wrap {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
  max-width: 360px;
  margin-left: 4px;
}

.search-icon {
  position: absolute;
  left: 9px;
  color: var(--text-tertiary);
  pointer-events: none;
}

.lib-search {
  width: 100%;
  height: 28px;
  padding: 0 28px 0 28px;
  border: 1px solid var(--border-medium);
  border-radius: var(--radius);
  font-family: var(--font-ui);
  font-size: 12px;
  color: var(--text);
  background: var(--surface-solid);
  outline: none;
  transition: border-color var(--t), box-shadow var(--t);
}

.lib-search:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-soft);
}

.search-clear {
  position: absolute;
  right: 7px;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-tertiary);
  display: flex;
  align-items: center;
  padding: 2px;
  border-radius: 2px;
  transition: color var(--t);
}
.search-clear:hover { color: var(--text); }

.toolbar-btn {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 5px;
  height: 28px;
  padding: 0 10px;
  background: none;
  border: 1px solid var(--border-medium);
  border-radius: var(--radius);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  color: var(--text-secondary);
  transition: background var(--t), color var(--t);
}
.toolbar-btn:hover:not(:disabled) { background: var(--bg-chrome-active); color: var(--text); }
.toolbar-btn:disabled { opacity: 0.5; cursor: not-allowed; }

.add-btn {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 5px;
  height: 28px;
  padding: 0 11px;
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

/* Filter bar */
.lib-filter-bar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 16px 8px 20px;
  flex-wrap: wrap;
}

.filter-pill {
  height: 22px;
  padding: 0 8px;
  border: 1px solid var(--border-medium);
  border-radius: 11px;
  font-family: var(--font-ui);
  font-size: 10.5px;
  font-weight: 600;
  letter-spacing: 0.02em;
  text-transform: uppercase;
  cursor: pointer;
  background: none;
  color: var(--text-tertiary);
  transition: background var(--t), color var(--t), border-color var(--t);
}
.filter-pill:hover {
  background: var(--bg-chrome-active);
  color: var(--text-secondary);
}

.filter-sep {
  flex: 0 0 1px;
  height: 14px;
  background: var(--border);
  margin: 0 4px;
}

.year-range {
  display: flex;
  align-items: center;
  gap: 4px;
}

.year-input {
  width: 52px;
  height: 22px;
  padding: 0 6px;
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  font-family: var(--font-ui);
  font-size: 11px;
  color: var(--text);
  background: none;
  outline: none;
  text-align: center;
  font-variant-numeric: tabular-nums;
  transition: border-color var(--t);
}
.year-input:focus { border-color: var(--accent); }

.year-dash {
  font-size: 11px;
  color: var(--text-tertiary);
}

.filter-clear-btn {
  margin-left: 4px;
  height: 22px;
  padding: 0 8px;
  border: none;
  border-radius: 11px;
  font-family: var(--font-ui);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  background: var(--bg-chrome-active);
  color: var(--text-secondary);
  transition: opacity var(--t);
}
.filter-clear-btn:hover { opacity: 0.8; }

/* Table */
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
  padding: 7px 12px;
  font-size: 10.5px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--text-tertiary);
  text-align: left;
  white-space: nowrap;
  user-select: none;
}

.lib-table th.sortable { cursor: pointer; }
.lib-table th.sortable:hover { color: var(--text-secondary); }
.lib-table th.sorted { color: var(--accent); }

.sort-icon { margin-left: 3px; font-size: 10px; }

.lib-row {
  border-bottom: 1px solid var(--border);
  cursor: pointer;
  transition: background var(--t);
}

.lib-row:hover { background: var(--bg-chrome); }
.lib-row.active { background: var(--accent-soft); }

.lib-table td {
  padding: 9px 12px;
  vertical-align: middle;
}

.col-type { width: 52px; }

.col-authors {
  width: 190px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 190px;
}

.col-year {
  width: 56px;
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
  text-align: right;
  padding-right: 16px;
}

.row-title {
  font-weight: 500;
  color: var(--text);
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
  line-height: 1.4;
}

.row-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 3px;
  margin-top: 4px;
}

.row-tag-chip {
  font-size: 9.5px;
  font-weight: 600;
  letter-spacing: 0.02em;
  padding: 1px 5px;
  border-radius: 3px;
  border: 1px solid transparent;
  white-space: nowrap;
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
  width: 280px;
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
  padding: 11px 14px 9px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.dp-label {
  font-size: 9.5px;
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
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  transition: background var(--t), color var(--t);
}
.dp-close:hover { background: var(--bg-chrome-active); color: var(--text); }

.dp-body {
  padding: 14px 14px 18px;
  overflow-y: auto;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 11px;
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
  font-size: 13.5px;
  font-weight: 700;
  line-height: 1.45;
  color: var(--text);
}

.dp-meta {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.dp-authors { font-size: 12px; font-weight: 500; color: var(--text); }
.dp-venue { font-size: 11px; color: var(--text-secondary); }

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
  gap: 5px 12px;
}

.dp-field {
  display: flex;
  align-items: baseline;
  gap: 4px;
  font-size: 11px;
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

/* Tags in detail panel */
.dp-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.dp-tag-chip {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 7px;
  border-radius: 4px;
  border: 1px solid transparent;
  white-space: nowrap;
}

/* Collections in detail panel */
.dp-coll-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 5px;
}

.dp-coll-chip {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  font-size: 11px;
  font-weight: 500;
  padding: 2px 6px 2px 8px;
  background: var(--bg-chrome);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
}

.dp-coll-remove {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-tertiary);
  font-size: 13px;
  line-height: 1;
  padding: 0;
  display: flex;
  align-items: center;
  transition: color var(--t);
}
.dp-coll-remove:hover { color: #C0392B; }

.dp-coll-add {
  display: flex;
  align-items: center;
  gap: 6px;
}

.dp-coll-select {
  flex: 1;
  height: 26px;
  padding: 0 6px;
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  font-family: var(--font-ui);
  font-size: 11.5px;
  color: var(--text-secondary);
  background: var(--bg);
  outline: none;
  cursor: pointer;
}

.dp-coll-add-btn {
  height: 26px;
  padding: 0 10px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius-sm);
  font-size: 11.5px;
  font-weight: 500;
  cursor: pointer;
  transition: opacity var(--t);
  flex-shrink: 0;
}
.dp-coll-add-btn:hover { opacity: 0.88; }

.dp-empty-hint {
  font-size: 11px;
  color: var(--text-tertiary);
  font-style: italic;
}

.dp-abstract {
  font-family: var(--font-doc);
  font-size: 12px;
  line-height: 1.65;
  color: var(--text-secondary);
}

.dp-keywords {
  font-size: 11.5px;
  color: var(--text-secondary);
  line-height: 1.5;
}

/* ── Detail panel actions ──────────────────────────────────────────────────── */

.dp-actions {
  display: flex;
  flex-direction: column;
  gap: 1px;
  padding: 7px 8px 9px;
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

.dp-action-btn:hover { background: var(--bg-chrome-active); color: var(--text); }
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

.fs-btn.secondary { background: var(--bg-chrome-active); color: var(--text-secondary); }
.fs-btn.secondary:hover { opacity: 0.8; }
.fs-btn.danger { background: #C0392B; color: #fff; }
.fs-btn.danger:hover { opacity: 0.88; }
.fs-btn.primary { background: var(--accent); color: #fff; }
.fs-btn.primary:hover { opacity: 0.88; }
.fs-btn.primary:disabled { opacity: 0.5; cursor: not-allowed; }

/* ── Import dialog ──────────────────────────────────────────────────────────── */

.import-dialog {
  max-width: 420px;
}

.import-file-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 10px 0 12px;
}

.import-path {
  flex: 1;
  font-family: var(--font-mono);
  font-size: 10.5px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  background: var(--bg-chrome);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 4px 8px;
  min-width: 0;
}

.import-mode {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 10px;
}

.mode-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12.5px;
  color: var(--text-secondary);
  cursor: pointer;
}

.mode-label input[type="radio"] {
  accent-color: var(--accent);
}

.import-result {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px;
  background: #16963F18;
  border-radius: var(--radius-sm);
  font-size: 12.5px;
  font-weight: 500;
}

.result-added   { color: #16963F; }
.result-skipped { color: var(--text-secondary); }
.result-errors  { color: var(--accent-orange); }

.fetch-error { font-size: 11.5px; color: var(--accent-orange); }

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

/* ── Files / Attachments ──────────────────────────────────────────────────── */

.dp-files-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.dp-attach-btn {
  background: none;
  border: 1px solid var(--border-medium);
  border-radius: 4px;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--text-secondary);
  padding: 0;
  transition: background var(--t);
}
.dp-attach-btn:hover:not(:disabled) { background: var(--bg-chrome); }
.dp-attach-btn:disabled { opacity: 0.4; cursor: default; }

.dp-file-row {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 4px 0;
  border-bottom: 1px solid var(--border);
}
.dp-file-row:last-child { border-bottom: none; }

.dp-file-icon { color: var(--text-tertiary); flex-shrink: 0; }

.dp-file-name {
  flex: 1;
  font-size: 11.5px;
  color: var(--accent);
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}
.dp-file-name:hover { text-decoration: underline; }

.dp-file-ext,
.dp-file-del {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-tertiary);
  font-size: 13px;
  padding: 0 2px;
  line-height: 1;
  flex-shrink: 0;
  transition: color var(--t);
}
.dp-file-ext:hover { color: var(--accent); }
.dp-file-del:hover { color: #E8650A; }
</style>
