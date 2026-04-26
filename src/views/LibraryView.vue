<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useLibrary, type LibraryItem } from '../composables/useLibrary'

const { items, loading, loadItems } = useLibrary()

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
      </div>
    </Transition>
  </div>
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
  align-items: baseline;
  gap: 10px;
  padding: 18px 24px 12px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

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
