<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { useLibrary } from '../composables/useLibrary'

const {
  items, collections, tags, filterQuery, applyFilter,
  createCollection, renameCollection, deleteCollection,
} = useLibrary()

const isAllItems = computed(() =>
  !filterQuery.value.collectionId && !filterQuery.value.tagName
)

function selectAll() {
  filterQuery.value = { ...filterQuery.value, collectionId: null, tagName: null }
  applyFilter()
}

function selectCollection(id: number) {
  filterQuery.value = { ...filterQuery.value, collectionId: id, tagName: null }
  applyFilter()
}

function selectTag(name: string) {
  const already = filterQuery.value.tagName === name
  filterQuery.value = { ...filterQuery.value, tagName: already ? null : name, collectionId: null }
  applyFilter()
}

// ── New collection ─────────────────────────────────────────────────────────────

const creatingNew = ref(false)
const newName = ref('')
const newInput = ref<HTMLInputElement | null>(null)

async function startNew() {
  creatingNew.value = true
  newName.value = ''
  await nextTick()
  newInput.value?.focus()
}

async function confirmNew() {
  const name = newName.value.trim()
  if (name) await createCollection(name)
  creatingNew.value = false
  newName.value = ''
}

function onNewKey(e: KeyboardEvent) {
  if (e.key === 'Enter') confirmNew()
  if (e.key === 'Escape') { creatingNew.value = false; newName.value = '' }
}

// ── Rename ─────────────────────────────────────────────────────────────────────

const renamingId = ref<number | null>(null)
const renameValue = ref('')
const renameInput = ref<HTMLInputElement | null>(null)

async function startRename(id: number, current: string, e: Event) {
  e.stopPropagation()
  renamingId.value = id
  renameValue.value = current
  await nextTick()
  renameInput.value?.focus()
  renameInput.value?.select()
}

async function confirmRename(id: number) {
  const name = renameValue.value.trim()
  if (name) await renameCollection(id, name)
  renamingId.value = null
}

function onRenameKey(e: KeyboardEvent, id: number) {
  if (e.key === 'Enter') confirmRename(id)
  if (e.key === 'Escape') renamingId.value = null
}

// ── Delete ─────────────────────────────────────────────────────────────────────

async function doDelete(id: number, e: Event) {
  e.stopPropagation()
  await deleteCollection(id)
}
</script>

<template>
  <aside class="coll-sidebar">
    <!-- All Items -->
    <div
      class="coll-row coll-all"
      :class="{ active: isAllItems }"
      @click="selectAll"
    >
      <svg class="coll-icon" width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <line x1="1" y1="3" x2="11" y2="3"/>
        <line x1="1" y1="6" x2="11" y2="6"/>
        <line x1="1" y1="9" x2="11" y2="9"/>
      </svg>
      <span class="coll-name">All Items</span>
      <span class="coll-count">{{ items.length }}</span>
    </div>

    <!-- Collections -->
    <div class="coll-section">
      <div class="coll-section-head">
        <span class="coll-section-label">Collections</span>
        <button class="coll-icon-btn" @click="startNew" title="New collection">
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <line x1="5" y1="1" x2="5" y2="9"/>
            <line x1="1" y1="5" x2="9" y2="5"/>
          </svg>
        </button>
      </div>

      <div
        v-for="col in collections"
        :key="col.id"
        class="coll-row"
        :class="{ active: filterQuery.collectionId === col.id }"
        @click="selectCollection(col.id)"
        @dblclick="startRename(col.id, col.name, $event)"
      >
        <svg class="coll-icon" width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
          <path d="M1 3.5C1 2.67 1.67 2 2.5 2h2L5.5 3.5h4c.83 0 1.5.67 1.5 1.5v4c0 .83-.67 1.5-1.5 1.5h-7C1.67 10.5 1 9.83 1 9V3.5Z"/>
        </svg>

        <template v-if="renamingId === col.id">
          <input
            ref="renameInput"
            v-model="renameValue"
            class="coll-inline-input"
            @blur="confirmRename(col.id)"
            @keydown="onRenameKey($event, col.id)"
            @click.stop
          />
        </template>
        <template v-else>
          <span class="coll-name">{{ col.name }}</span>
          <span class="coll-count">{{ col.itemCount }}</span>
          <button
            class="coll-del-btn"
            @click="doDelete(col.id, $event)"
            title="Delete collection"
          >
            <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
              <line x1="1" y1="1" x2="8" y2="8"/>
              <line x1="8" y1="1" x2="1" y2="8"/>
            </svg>
          </button>
        </template>
      </div>

      <!-- New collection input row -->
      <div class="coll-row coll-new-row" v-if="creatingNew">
        <svg class="coll-icon" width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
          <path d="M1 3.5C1 2.67 1.67 2 2.5 2h2L5.5 3.5h4c.83 0 1.5.67 1.5 1.5v4c0 .83-.67 1.5-1.5 1.5h-7C1.67 10.5 1 9.83 1 9V3.5Z"/>
        </svg>
        <input
          ref="newInput"
          v-model="newName"
          class="coll-inline-input"
          placeholder="Collection name"
          @blur="confirmNew"
          @keydown="onNewKey"
        />
      </div>

      <div class="coll-empty-hint" v-if="collections.length === 0 && !creatingNew">
        No collections yet
      </div>
    </div>

    <!-- Tags -->
    <div class="coll-section" v-if="tags.length > 0">
      <div class="coll-section-head">
        <span class="coll-section-label">Tags</span>
      </div>
      <div
        v-for="tag in tags"
        :key="tag.id"
        class="coll-row coll-tag-row"
        :class="{ active: filterQuery.tagName === tag.name }"
        @click="selectTag(tag.name)"
      >
        <span class="tag-dot" :style="{ background: tag.color }"></span>
        <span class="coll-name">{{ tag.name }}</span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.coll-sidebar {
  width: 180px;
  flex-shrink: 0;
  border-right: 1px solid var(--border);
  background: var(--bg-chrome);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  padding-bottom: 16px;
}

.coll-section {
  margin-top: 6px;
}

.coll-section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px 3px 14px;
}

.coll-section-label {
  font-size: 9.5px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  color: var(--text-tertiary);
}

.coll-icon-btn {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-tertiary);
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-xs);
  transition: background var(--t), color var(--t);
}
.coll-icon-btn:hover {
  background: var(--bg-chrome-active);
  color: var(--text);
}

.coll-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 8px 5px 12px;
  cursor: pointer;
  border-radius: var(--radius-sm);
  margin: 0 4px;
  min-height: 27px;
  position: relative;
  transition: background var(--t);
}

.coll-row:hover {
  background: var(--bg-chrome-active);
}

.coll-row.active {
  background: var(--accent-soft);
}

.coll-row:hover .coll-del-btn {
  opacity: 1;
}

.coll-all {
  margin-top: 8px;
}

.coll-icon {
  flex-shrink: 0;
  color: var(--text-tertiary);
}

.coll-name {
  flex: 1;
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.coll-row.active .coll-name {
  color: var(--accent);
  font-weight: 500;
}

.coll-count {
  font-size: 10.5px;
  color: var(--text-tertiary);
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}

.coll-del-btn {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-tertiary);
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-xs);
  flex-shrink: 0;
  opacity: 0;
  transition: opacity var(--t), background var(--t), color var(--t);
}
.coll-del-btn:hover {
  background: rgba(192, 57, 43, 0.1);
  color: #C0392B;
}

.coll-inline-input {
  flex: 1;
  height: 22px;
  padding: 0 6px;
  border: 1px solid var(--accent);
  border-radius: var(--radius-xs);
  font-family: var(--font-ui);
  font-size: 12px;
  color: var(--text);
  background: var(--surface-solid);
  outline: none;
  box-shadow: 0 0 0 2px var(--accent-soft);
  min-width: 0;
}

.coll-empty-hint {
  padding: 4px 14px 6px;
  font-size: 11px;
  color: var(--text-tertiary);
  font-style: italic;
}

.coll-tag-row {
  padding-left: 12px;
}

.tag-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
</style>
