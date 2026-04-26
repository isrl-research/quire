<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useLibrary, type LibraryItem, type ItemInput } from '../composables/useLibrary'

// ── Fetch types ───────────────────────────────────────────────────────────────

interface FetchedMetadata {
  title?:        string
  authors?:      string
  year?:         string
  journal?:      string
  volume?:       string
  issue?:        string
  pages?:        string
  doi?:          string
  issn?:         string
  publisher?:    string
  url?:          string
  abstractText?: string
  booktitle?:    string
  isbn?:         string
  entryType?:    string
}

const { createItem, updateItem, tags, displayItems, createTag, setItemTags } = useLibrary()

const props = defineProps<{
  editItem?: LibraryItem | null
}>()

const emit = defineEmits<{
  close: []
  saved: [item: LibraryItem]
}>()

// ── Form state ────────────────────────────────────────────────────────────────

const entryType = ref('article')
const key       = ref('')
const title     = ref('')
const authors   = ref('')
const year      = ref('')
const journal   = ref('')
const booktitle = ref('')
const volume    = ref('')
const number    = ref('')
const pages     = ref('')
const publisher = ref('')
const institution = ref('')
const edition   = ref('')
const doi       = ref('')
const url       = ref('')
const isbn      = ref('')
const issn      = ref('')
const abstractText = ref('')
const keywords  = ref('')
const note      = ref('')

const saving   = ref(false)
const keyError = ref('')
let autoKey    = '' // tracks last auto-generated key to detect user override

// ── Tags ──────────────────────────────────────────────────────────────────────

const selectedTagIds = ref<number[]>([])
const newTagInput = ref('')
const TAG_COLORS = ['#0A5FBF', '#16963F', '#E8650A', '#7C3AED', '#C0392B', '#A5A4A2']

watch(tags, (newTags) => {
  if (props.editItem && selectedTagIds.value.length === 0 && props.editItem.tags.length > 0) {
    selectedTagIds.value = newTags
      .filter(t => props.editItem!.tags.includes(t.name))
      .map(t => t.id)
  }
}, { immediate: true })

function toggleTag(id: number) {
  const idx = selectedTagIds.value.indexOf(id)
  if (idx === -1) selectedTagIds.value = [...selectedTagIds.value, id]
  else selectedTagIds.value = selectedTagIds.value.filter(i => i !== id)
}

async function addNewTag() {
  const name = newTagInput.value.trim()
  if (!name) return
  const existing = tags.value.find(t => t.name.toLowerCase() === name.toLowerCase())
  if (existing) {
    if (!selectedTagIds.value.includes(existing.id)) {
      selectedTagIds.value = [...selectedTagIds.value, existing.id]
    }
  } else {
    const color = TAG_COLORS[tags.value.length % TAG_COLORS.length]
    const created = await createTag(name, color)
    selectedTagIds.value = [...selectedTagIds.value, created.id]
  }
  newTagInput.value = ''
}

function onTagInputKey(e: KeyboardEvent) {
  if (e.key === 'Enter' || e.key === ',') {
    e.preventDefault()
    addNewTag()
  }
}

function fillFrom(item: LibraryItem) {
  entryType.value    = item.entryType
  key.value          = item.key
  title.value        = item.title       ?? ''
  authors.value      = item.authors     ?? ''
  year.value         = item.year        ?? ''
  journal.value      = item.journal     ?? ''
  booktitle.value    = item.booktitle   ?? ''
  volume.value       = item.volume      ?? ''
  number.value       = item.number      ?? ''
  pages.value        = item.pages       ?? ''
  publisher.value    = item.publisher   ?? ''
  institution.value  = item.institution ?? ''
  edition.value      = item.edition     ?? ''
  doi.value          = item.doi         ?? ''
  url.value          = item.url         ?? ''
  isbn.value         = item.isbn        ?? ''
  issn.value         = item.issn        ?? ''
  abstractText.value = item.abstractText ?? ''
  keywords.value     = item.keywords    ?? ''
  note.value         = item.note        ?? ''
}

// Pre-fill when editing
if (props.editItem) {
  fillFrom(props.editItem)
  autoKey = ''
} else {
  autoKey = ''
}

// ── Auto-key generation ───────────────────────────────────────────────────────

function makeAutoKey(authorsStr: string, yearStr: string): string {
  const firstAuthor = authorsStr.split(/[,&]| and /i)[0].trim()
  const parts = firstAuthor.split(/\s+/)
  const lastName = parts[parts.length - 1] ?? firstAuthor
  const cleaned = lastName.toLowerCase().replace(/[^a-z0-9]/g, '')
  return cleaned && yearStr ? `${cleaned}${yearStr}` : ''
}

watch([authors, year], () => {
  if (props.editItem) return  // don't auto-generate when editing
  const candidate = makeAutoKey(authors.value, year.value)
  if (!candidate) return
  // Only update if key is empty or was last auto-set by us
  if (key.value === '' || key.value === autoKey) {
    autoKey = candidate
    key.value = candidate
  }
})

// ── Field visibility per type ─────────────────────────────────────────────────

const showJournal     = computed(() => entryType.value === 'article')
const showBooktitle   = computed(() => entryType.value === 'inproceedings')
const showVolNum      = computed(() => ['article', 'book'].includes(entryType.value))
const showPages       = computed(() => ['article', 'inproceedings'].includes(entryType.value))
const showPublisher   = computed(() => ['book', 'misc'].includes(entryType.value))
const showInstitution = computed(() => entryType.value === 'techreport')
const showEdition     = computed(() => entryType.value === 'book')
const showIsbn        = computed(() => entryType.value === 'book')
const showIssn        = computed(() => entryType.value === 'article')

const isEdit = computed(() => !!props.editItem)
const formTitle = computed(() => isEdit.value ? 'Edit Entry' : 'Add Entry')

// ── Save ──────────────────────────────────────────────────────────────────────

async function save() {
  keyError.value = ''
  if (!key.value.trim()) {
    keyError.value = 'Key is required'
    return
  }

  const input: ItemInput = {
    key:          key.value.trim(),
    entryType:    entryType.value,
    title:        title.value      || undefined,
    authors:      authors.value    || undefined,
    year:         year.value       || undefined,
    journal:      journal.value    || undefined,
    booktitle:    booktitle.value  || undefined,
    volume:       volume.value     || undefined,
    number:       number.value     || undefined,
    pages:        pages.value      || undefined,
    publisher:    publisher.value  || undefined,
    institution:  institution.value || undefined,
    edition:      edition.value    || undefined,
    doi:          doi.value        || undefined,
    url:          url.value        || undefined,
    isbn:         isbn.value       || undefined,
    issn:         issn.value       || undefined,
    abstractText: abstractText.value || undefined,
    keywords:     keywords.value   || undefined,
    note:         note.value       || undefined,
  }

  saving.value = true
  try {
    const result = isEdit.value
      ? await updateItem(props.editItem!.id, input)
      : await createItem(input)
    await setItemTags(result.id, selectedTagIds.value)
    const freshItem = displayItems.value.find(i => i.id === result.id) ?? result
    emit('saved', freshItem)
  } catch (e: unknown) {
    const msg = String(e)
    if (msg.includes('UNIQUE') || msg.includes('unique')) {
      keyError.value = 'A key with that name already exists'
    } else {
      keyError.value = msg
    }
  } finally {
    saving.value = false
  }
}

// ── Metadata fetch ────────────────────────────────────────────────────────────

const fetchId      = ref('')
const fetching     = ref(false)
const fetchError   = ref('')
const fetchedMeta  = ref<FetchedMetadata | null>(null)
const fetchSource  = ref('')

function stripDoiUrl(s: string): string {
  return s.replace(/^https?:\/\/(dx\.)?doi\.org\//i, '').replace(/^doi:/i, '').trim()
}

function detectIdType(id: string): 'doi' | 'arxiv' | 'isbn' {
  const s = stripDoiUrl(id.trim())
  if (/^10\.\d{4}/.test(s)) return 'doi'
  if (/^(arXiv:|arxiv:)?\d{4}\.\d{4,5}(v\d+)?$/.test(s)) return 'arxiv'
  if (/^(arXiv:|arxiv:)?[a-z-]+\/\d+$/i.test(s)) return 'arxiv'
  const digits = s.replace(/[^0-9X]/gi, '')
  if (digits.length === 10 || digits.length === 13) return 'isbn'
  return 'doi'
}

async function fetchMetadata() {
  const id = fetchId.value.trim()
  if (!id) return
  fetching.value = true
  fetchError.value = ''
  fetchedMeta.value = null
  try {
    const type = detectIdType(id)
    if (type === 'doi') {
      fetchSource.value = 'CrossRef'
      fetchedMeta.value = await invoke<FetchedMetadata>('fetch_doi_metadata', { doi: id })
    } else if (type === 'arxiv') {
      fetchSource.value = 'arXiv'
      fetchedMeta.value = await invoke<FetchedMetadata>('fetch_arxiv_metadata', { arxivId: id })
    } else {
      fetchSource.value = 'OpenLibrary'
      fetchedMeta.value = await invoke<FetchedMetadata>('fetch_isbn_metadata', { isbn: id })
    }
  } catch (e) {
    fetchError.value = String(e)
  } finally {
    fetching.value = false
  }
}

const FETCH_FIELD_LABELS: Partial<Record<keyof FetchedMetadata, string>> = {
  title: 'Title', authors: 'Authors', year: 'Year', journal: 'Journal',
  volume: 'Volume', issue: 'Issue No.', pages: 'Pages', doi: 'DOI',
  issn: 'ISSN', publisher: 'Publisher', url: 'URL', abstractText: 'Abstract',
  booktitle: 'Conference', isbn: 'ISBN', entryType: 'Type',
}

const fetchedFields = computed(() => {
  if (!fetchedMeta.value) return []
  return (Object.keys(fetchedMeta.value) as (keyof FetchedMetadata)[])
    .filter(k => fetchedMeta.value![k] !== undefined && FETCH_FIELD_LABELS[k])
    .map(k => ({ key: k, label: FETCH_FIELD_LABELS[k]!, value: fetchedMeta.value![k]! }))
})

function applyFetchField(key: keyof FetchedMetadata) {
  const val = fetchedMeta.value?.[key]
  if (val === undefined) return
  switch (key) {
    case 'title':        title.value        = val; break
    case 'authors':      authors.value      = val; break
    case 'year':         year.value         = val; break
    case 'journal':      journal.value      = val; break
    case 'volume':       volume.value       = val; break
    case 'issue':        number.value       = val; break
    case 'pages':        pages.value        = val; break
    case 'doi':          doi.value          = val; break
    case 'issn':         issn.value         = val; break
    case 'publisher':    publisher.value    = val; break
    case 'url':          url.value          = val; break
    case 'abstractText': abstractText.value = val; break
    case 'booktitle':    booktitle.value    = val; break
    case 'isbn':         isbn.value         = val; break
    case 'entryType':    entryType.value    = val; break
  }
}

function applyAllFetched() {
  if (!fetchedMeta.value) return
  for (const key of Object.keys(fetchedMeta.value) as (keyof FetchedMetadata)[]) {
    applyFetchField(key)
  }
  fetchedMeta.value = null
}

function onBackdropClick(e: MouseEvent) {
  if (e.target === e.currentTarget) emit('close')
}
</script>

<template>
  <Teleport to="body">
    <div class="form-backdrop" @click="onBackdropClick">
      <div class="form-sheet" role="dialog" :aria-label="formTitle">

        <!-- Header -->
        <div class="fs-header">
          <span class="fs-title">{{ formTitle }}</span>
          <button class="fs-close" @click="emit('close')" title="Close">
            <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round">
              <line x1="1" y1="1" x2="12" y2="12"/>
              <line x1="12" y1="1" x2="1" y2="12"/>
            </svg>
          </button>
        </div>

        <!-- Body -->
        <div class="fs-body">

          <!-- Metadata fetch -->
          <div class="fetch-row">
            <input
              v-model="fetchId"
              class="fs-input fetch-input"
              placeholder="DOI, arXiv ID, or ISBN…"
              @keydown.enter="fetchMetadata"
            />
            <button
              class="fetch-btn"
              :disabled="!fetchId.trim() || fetching"
              @click="fetchMetadata"
              type="button"
            >{{ fetching ? '…' : 'Fetch' }}</button>
          </div>

          <!-- Fetch error -->
          <p class="fetch-error" v-if="fetchError">{{ fetchError }}</p>

          <!-- Fetch result diff panel -->
          <div class="fetch-panel" v-if="fetchedMeta && fetchedFields.length">
            <div class="fetch-panel-head">
              <span class="fetch-panel-source">From {{ fetchSource }}</span>
              <div class="fetch-panel-actions">
                <button class="fetch-apply-all" @click="applyAllFetched" type="button">Apply All</button>
                <button class="fetch-dismiss" @click="fetchedMeta = null" type="button">Dismiss</button>
              </div>
            </div>
            <div class="fetch-fields">
              <div v-for="f in fetchedFields" :key="f.key" class="fetch-field">
                <span class="fetch-field-label">{{ f.label }}</span>
                <span class="fetch-field-value">{{ f.value.length > 80 ? f.value.slice(0, 80) + '…' : f.value }}</span>
                <button class="fetch-field-apply" @click="applyFetchField(f.key)" type="button">↓</button>
              </div>
            </div>
          </div>

          <!-- Entry type row -->
          <div class="fs-field-row">
            <label class="fs-label">Type</label>
            <div class="type-tabs">
              <button
                v-for="t in ['article','book','inproceedings','techreport','misc']"
                :key="t"
                class="type-tab"
                :class="{ active: entryType === t }"
                @click="entryType = t"
              >{{ t }}</button>
            </div>
          </div>

          <!-- Key -->
          <div class="fs-field-row" :class="{ error: keyError }">
            <label class="fs-label" for="ef-key">Key <span class="required">*</span></label>
            <div class="fs-input-wrap">
              <input
                id="ef-key"
                v-model="key"
                class="fs-input mono"
                placeholder="e.g. smith2024"
                autocomplete="off"
                spellcheck="false"
                @input="keyError = ''"
              />
              <span class="fs-error" v-if="keyError">{{ keyError }}</span>
            </div>
          </div>

          <!-- Title -->
          <div class="fs-field-row">
            <label class="fs-label" for="ef-title">Title</label>
            <input id="ef-title" v-model="title" class="fs-input" placeholder="Full title of the work" />
          </div>

          <!-- Authors -->
          <div class="fs-field-row">
            <label class="fs-label" for="ef-authors">Authors</label>
            <input id="ef-authors" v-model="authors" class="fs-input" placeholder="Last, First and Last, First" />
          </div>

          <!-- Year -->
          <div class="fs-field-row">
            <label class="fs-label" for="ef-year">Year</label>
            <input id="ef-year" v-model="year" class="fs-input short" placeholder="2024" maxlength="4" />
          </div>

          <!-- Journal (article only) -->
          <div class="fs-field-row" v-if="showJournal">
            <label class="fs-label" for="ef-journal">Journal</label>
            <input id="ef-journal" v-model="journal" class="fs-input" placeholder="Journal name" />
          </div>

          <!-- Booktitle (inproceedings only) -->
          <div class="fs-field-row" v-if="showBooktitle">
            <label class="fs-label" for="ef-booktitle">Conference</label>
            <input id="ef-booktitle" v-model="booktitle" class="fs-input" placeholder="Proceedings of..." />
          </div>

          <!-- Publisher (book, misc) -->
          <div class="fs-field-row" v-if="showPublisher">
            <label class="fs-label" for="ef-publisher">Publisher</label>
            <input id="ef-publisher" v-model="publisher" class="fs-input" placeholder="Publisher name" />
          </div>

          <!-- Institution (techreport) -->
          <div class="fs-field-row" v-if="showInstitution">
            <label class="fs-label" for="ef-inst">Institution</label>
            <input id="ef-inst" v-model="institution" class="fs-input" placeholder="Issuing institution" />
          </div>

          <!-- Volume + Number (article, book) -->
          <div class="fs-field-row" v-if="showVolNum">
            <label class="fs-label">Vol / No.</label>
            <div class="fs-inline">
              <input v-model="volume" class="fs-input short" placeholder="Vol" />
              <input v-model="number" class="fs-input short" placeholder="No." />
            </div>
          </div>

          <!-- Report number (techreport) -->
          <div class="fs-field-row" v-if="entryType === 'techreport'">
            <label class="fs-label" for="ef-number">Report No.</label>
            <input id="ef-number" v-model="number" class="fs-input short" placeholder="Report number" />
          </div>

          <!-- Pages -->
          <div class="fs-field-row" v-if="showPages">
            <label class="fs-label" for="ef-pages">Pages</label>
            <input id="ef-pages" v-model="pages" class="fs-input short" placeholder="100–115" />
          </div>

          <!-- Edition (book) -->
          <div class="fs-field-row" v-if="showEdition">
            <label class="fs-label" for="ef-edition">Edition</label>
            <input id="ef-edition" v-model="edition" class="fs-input short" placeholder="2nd" />
          </div>

          <!-- DOI -->
          <div class="fs-field-row">
            <label class="fs-label" for="ef-doi">DOI</label>
            <input id="ef-doi" v-model="doi" class="fs-input mono" placeholder="10.xxxx/..." />
          </div>

          <!-- ISBN (book) -->
          <div class="fs-field-row" v-if="showIsbn">
            <label class="fs-label" for="ef-isbn">ISBN</label>
            <input id="ef-isbn" v-model="isbn" class="fs-input mono short" placeholder="978-..." />
          </div>

          <!-- ISSN (article) -->
          <div class="fs-field-row" v-if="showIssn">
            <label class="fs-label" for="ef-issn">ISSN</label>
            <input id="ef-issn" v-model="issn" class="fs-input mono short" placeholder="1234-5678" />
          </div>

          <!-- URL -->
          <div class="fs-field-row">
            <label class="fs-label" for="ef-url">URL</label>
            <input id="ef-url" v-model="url" class="fs-input" placeholder="https://..." />
          </div>

          <!-- Abstract -->
          <div class="fs-field-row fs-field-tall">
            <label class="fs-label" for="ef-abs">Abstract</label>
            <textarea id="ef-abs" v-model="abstractText" class="fs-textarea" placeholder="Abstract text…" rows="4" />
          </div>

          <!-- Keywords -->
          <div class="fs-field-row">
            <label class="fs-label" for="ef-kw">Keywords</label>
            <input id="ef-kw" v-model="keywords" class="fs-input" placeholder="keyword1, keyword2" />
          </div>

          <!-- Note -->
          <div class="fs-field-row">
            <label class="fs-label" for="ef-note">Note</label>
            <input id="ef-note" v-model="note" class="fs-input" placeholder="Miscellaneous note" />
          </div>

          <!-- Tags -->
          <div class="fs-field-row fs-field-tags">
            <label class="fs-label">Tags</label>
            <div class="tag-section">
              <div class="tag-chips" v-if="tags.length > 0">
                <button
                  v-for="tag in tags"
                  :key="tag.id"
                  class="tag-chip-btn"
                  :class="{ selected: selectedTagIds.includes(tag.id) }"
                  :style="selectedTagIds.includes(tag.id) ? {
                    background: tag.color + '22',
                    color: tag.color,
                    borderColor: tag.color + '77',
                  } : {}"
                  @click="toggleTag(tag.id)"
                  type="button"
                >{{ tag.name }}</button>
              </div>
              <div class="tag-add-row">
                <input
                  v-model="newTagInput"
                  class="fs-input tag-input"
                  placeholder="New tag name…"
                  @keydown="onTagInputKey"
                />
                <button
                  class="tag-add-confirm"
                  type="button"
                  @click="addNewTag"
                  :disabled="!newTagInput.trim()"
                >Add</button>
              </div>
            </div>
          </div>

        </div>

        <!-- Footer -->
        <div class="fs-footer">
          <button class="fs-btn secondary" @click="emit('close')">Cancel</button>
          <button class="fs-btn primary" @click="save" :disabled="saving">
            {{ saving ? 'Saving…' : (isEdit ? 'Save Changes' : 'Add Entry') }}
          </button>
        </div>

      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.form-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.32);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.form-sheet {
  background: var(--surface-solid);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  width: 100%;
  max-width: 540px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Header */
.fs-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px 12px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.fs-title {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text);
}

.fs-close {
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
.fs-close:hover {
  background: var(--bg-chrome-active);
  color: var(--text);
}

/* Body */
.fs-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px 4px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.fs-field-row {
  display: grid;
  grid-template-columns: 84px 1fr;
  align-items: center;
  gap: 10px;
  min-height: 32px;
}

.fs-field-row.fs-field-tall {
  align-items: flex-start;
  padding-top: 4px;
}

.fs-field-row.error .fs-input {
  border-color: var(--accent-orange);
}

.fs-label {
  font-size: 11.5px;
  font-weight: 500;
  color: var(--text-secondary);
  text-align: right;
  flex-shrink: 0;
}

.required {
  color: var(--accent-orange);
}

.fs-input-wrap {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.fs-error {
  font-size: 11px;
  color: var(--accent-orange);
}

.fs-input {
  width: 100%;
  height: 30px;
  padding: 0 9px;
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  font-family: var(--font-ui);
  font-size: 12.5px;
  color: var(--text);
  background: var(--bg);
  outline: none;
  transition: border-color var(--t), box-shadow var(--t);
}

.fs-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2.5px var(--accent-soft);
}

.fs-input.mono {
  font-family: var(--font-mono);
  font-size: 11.5px;
}

.fs-input.short {
  width: auto;
  min-width: 80px;
  max-width: 140px;
}

.fs-inline {
  display: flex;
  gap: 8px;
}

.fs-textarea {
  width: 100%;
  padding: 7px 9px;
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  font-family: var(--font-doc);
  font-size: 12.5px;
  color: var(--text);
  background: var(--bg);
  outline: none;
  resize: vertical;
  line-height: 1.55;
  transition: border-color var(--t), box-shadow var(--t);
}

.fs-textarea:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2.5px var(--accent-soft);
}

/* Type tabs */
.type-tabs {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.type-tab {
  height: 26px;
  padding: 0 10px;
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  font-size: 11.5px;
  font-weight: 500;
  cursor: pointer;
  background: var(--bg);
  color: var(--text-secondary);
  transition: background var(--t), color var(--t), border-color var(--t);
}

.type-tab:hover {
  background: var(--bg-chrome-active);
  color: var(--text);
}

.type-tab.active {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}

/* Footer */
.fs-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px 14px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.fs-btn {
  height: 32px;
  padding: 0 16px;
  border-radius: var(--radius);
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: opacity var(--t), background var(--t);
}

.fs-btn.secondary {
  background: var(--bg-chrome-active);
  color: var(--text-secondary);
}
.fs-btn.secondary:hover { opacity: 0.8; }

.fs-btn.primary {
  background: var(--accent);
  color: #fff;
}
.fs-btn.primary:hover { opacity: 0.88; }
.fs-btn.primary:disabled { opacity: 0.5; cursor: not-allowed; }

/* Tags section */
.fs-field-tags {
  align-items: flex-start;
  padding-top: 4px;
}

.tag-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}

.tag-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.tag-chip-btn {
  height: 24px;
  padding: 0 9px;
  border: 1px solid var(--border-medium);
  border-radius: 12px;
  font-family: var(--font-ui);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  background: none;
  color: var(--text-secondary);
  transition: background var(--t), color var(--t), border-color var(--t);
}

.tag-chip-btn:hover {
  background: var(--bg-chrome-active);
  color: var(--text);
}

.tag-add-row {
  display: flex;
  gap: 6px;
  align-items: center;
}

.tag-input {
  flex: 1;
  height: 28px;
}

.tag-add-confirm {
  height: 28px;
  padding: 0 10px;
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  font-family: var(--font-ui);
  font-size: 11.5px;
  font-weight: 500;
  cursor: pointer;
  background: var(--bg-chrome-active);
  color: var(--text-secondary);
  transition: opacity var(--t);
  flex-shrink: 0;
}

.tag-add-confirm:hover { opacity: 0.8; }
.tag-add-confirm:disabled { opacity: 0.4; cursor: not-allowed; }

/* ── Metadata fetch ──────────────────────────────────────────────────────────── */

.fetch-row {
  display: flex;
  gap: 6px;
  align-items: center;
}

.fetch-input {
  flex: 1;
  font-family: var(--font-mono);
  font-size: 11.5px;
}

.fetch-btn {
  height: 30px;
  padding: 0 12px;
  background: var(--bg-chrome-active);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  font-family: var(--font-ui);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  color: var(--text-secondary);
  transition: background var(--t), opacity var(--t);
  flex-shrink: 0;
}
.fetch-btn:hover:not(:disabled) { background: var(--accent); color: #fff; border-color: var(--accent); }
.fetch-btn:disabled { opacity: 0.45; cursor: not-allowed; }

.fetch-error {
  font-size: 11.5px;
  color: var(--accent-orange);
  padding: 2px 0;
}

.fetch-panel {
  border: 1px solid var(--border-medium);
  border-radius: var(--radius);
  background: var(--bg-chrome);
  overflow: hidden;
}

.fetch-panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 7px 10px 6px;
  border-bottom: 1px solid var(--border);
}

.fetch-panel-source {
  font-size: 10.5px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.07em;
  color: var(--accent);
}

.fetch-panel-actions {
  display: flex;
  gap: 6px;
}

.fetch-apply-all {
  height: 24px;
  padding: 0 10px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity var(--t);
}
.fetch-apply-all:hover { opacity: 0.88; }

.fetch-dismiss {
  height: 24px;
  padding: 0 8px;
  background: none;
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  font-size: 11px;
  cursor: pointer;
  color: var(--text-tertiary);
  transition: background var(--t);
}
.fetch-dismiss:hover { background: var(--bg-chrome-active); }

.fetch-fields {
  padding: 4px 0;
  max-height: 180px;
  overflow-y: auto;
}

.fetch-field {
  display: grid;
  grid-template-columns: 72px 1fr 24px;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
}

.fetch-field-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-tertiary);
  text-align: right;
}

.fetch-field-value {
  font-size: 11.5px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fetch-field-apply {
  background: none;
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-xs);
  font-size: 11px;
  cursor: pointer;
  color: var(--accent);
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background var(--t);
}
.fetch-field-apply:hover { background: var(--accent-soft); }
</style>
