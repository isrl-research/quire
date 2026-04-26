import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// ── Types ─────────────────────────────────────────────────────────────────────

export interface LibraryItem {
  id: number
  key: string
  entryType: string
  title?: string
  authors?: string
  year?: string
  journal?: string
  doi?: string
  abstractText?: string
  url?: string
  volume?: string
  issue?: string
  pages?: string
  publisher?: string
  booktitle?: string
  edition?: string
  month?: string
  keywords?: string
  note?: string
  isbn?: string
  issn?: string
  number?: string
  institution?: string
  addedAt: number
  updatedAt: number
  tags: string[]
}

export interface ItemInput {
  key: string
  entryType: string
  title?: string
  authors?: string
  year?: string
  journal?: string
  doi?: string
  abstractText?: string
  url?: string
  volume?: string
  issue?: string
  pages?: string
  publisher?: string
  booktitle?: string
  edition?: string
  month?: string
  keywords?: string
  note?: string
  isbn?: string
  issn?: string
  number?: string
  institution?: string
}

export interface ImportResult {
  added:   number
  skipped: number
  errors:  string[]
}

export interface SearchQuery {
  text?: string
  entryTypes?: string[]
  yearMin?: string
  yearMax?: string
  hasDoi?: boolean
  collectionId?: number | null
  tagName?: string | null
}

export interface Collection {
  id: number
  name: string
  parentId?: number | null
  createdAt: number
  itemCount: number
}

export interface Tag {
  id: number
  name: string
  color: string
}

export interface Attachment {
  id: number
  itemId: number
  fileName: string
  filePath: string
  addedAt: number
}

// ── Singletons ────────────────────────────────────────────────────────────────

const items        = ref<LibraryItem[]>([])   // full unfiltered list
const displayItems = ref<LibraryItem[]>([])   // currently shown (filtered)
const loading      = ref(false)
const error        = ref<string | null>(null)
const collections  = ref<Collection[]>([])
const tags         = ref<Tag[]>([])
const filterQuery  = ref<SearchQuery>({})

// ── Composable ────────────────────────────────────────────────────────────────

export function useLibrary() {

  // ── Item loading / filtering ────────────────────────────────────────────────

  async function loadItems(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      items.value = await invoke<LibraryItem[]>('get_library_items')
      displayItems.value = [...items.value]
    } catch (e) {
      error.value = String(e)
      console.error('[library] loadItems:', e)
    } finally {
      loading.value = false
    }
  }

  function isQueryEmpty(q: SearchQuery): boolean {
    return !q.text && !q.collectionId && !q.tagName && !q.hasDoi &&
           (!q.entryTypes || q.entryTypes.length === 0) && !q.yearMin && !q.yearMax
  }

  async function applyFilter(): Promise<void> {
    if (isQueryEmpty(filterQuery.value)) {
      displayItems.value = [...items.value]
      return
    }
    try {
      displayItems.value = await invoke<LibraryItem[]>('search_library_items', {
        query: filterQuery.value,
      })
    } catch (e) {
      console.error('[library] applyFilter:', e)
    }
  }

  async function refreshAndFilter(): Promise<void> {
    items.value = await invoke<LibraryItem[]>('get_library_items')
    await applyFilter()
  }

  // ── CRUD ────────────────────────────────────────────────────────────────────

  async function createItem(input: ItemInput): Promise<LibraryItem> {
    const created = await invoke<LibraryItem>('create_library_item', { item: input })
    await refreshAndFilter()
    return created
  }

  async function updateItem(id: number, input: ItemInput): Promise<LibraryItem> {
    const updated = await invoke<LibraryItem>('update_library_item', { id, item: input })
    await refreshAndFilter()
    return updated
  }

  async function deleteItem(id: number): Promise<void> {
    await invoke('delete_library_item', { id })
    await refreshAndFilter()
  }

  async function searchItems(query: SearchQuery): Promise<LibraryItem[]> {
    try {
      return await invoke<LibraryItem[]>('search_library_items', { query })
    } catch (e) {
      console.error('[library] searchItems:', e)
      return []
    }
  }

  // ── Collections ─────────────────────────────────────────────────────────────

  async function loadCollections(): Promise<void> {
    try {
      collections.value = await invoke<Collection[]>('get_collections')
    } catch (e) {
      console.error('[library] loadCollections:', e)
    }
  }

  async function createCollection(name: string, parentId?: number): Promise<Collection> {
    const col = await invoke<Collection>('create_collection', { name, parentId: parentId ?? null })
    collections.value.push(col)
    return col
  }

  async function renameCollection(id: number, name: string): Promise<void> {
    const updated = await invoke<Collection>('rename_collection', { id, name })
    const idx = collections.value.findIndex(c => c.id === id)
    if (idx !== -1) collections.value[idx] = updated
  }

  async function deleteCollection(id: number): Promise<void> {
    await invoke('delete_collection', { id })
    collections.value = collections.value.filter(c => c.id !== id)
    if (filterQuery.value.collectionId === id) {
      filterQuery.value = { ...filterQuery.value, collectionId: null }
      await applyFilter()
    }
  }

  async function addItemToCollection(collectionId: number, itemId: number): Promise<void> {
    await invoke('add_item_to_collection', { collectionId, itemId })
    await loadCollections()
  }

  async function removeItemFromCollection(collectionId: number, itemId: number): Promise<void> {
    await invoke('remove_item_from_collection', { collectionId, itemId })
    await loadCollections()
  }

  async function getItemCollectionIds(itemId: number): Promise<number[]> {
    return invoke<number[]>('get_item_collection_ids', { itemId })
  }

  // ── Tags ────────────────────────────────────────────────────────────────────

  async function loadTags(): Promise<void> {
    try {
      tags.value = await invoke<Tag[]>('get_tags')
    } catch (e) {
      console.error('[library] loadTags:', e)
    }
  }

  async function createTag(name: string, color: string): Promise<Tag> {
    const tag = await invoke<Tag>('create_tag', { name, color })
    tags.value.push(tag)
    tags.value.sort((a, b) => a.name.localeCompare(b.name))
    return tag
  }

  async function updateTagColor(id: number, color: string): Promise<void> {
    const updated = await invoke<Tag>('update_tag_color', { id, color })
    const idx = tags.value.findIndex(t => t.id === id)
    if (idx !== -1) tags.value[idx] = updated
  }

  async function deleteTag(id: number): Promise<void> {
    await invoke('delete_tag', { id })
    tags.value = tags.value.filter(t => t.id !== id)
    if (filterQuery.value.tagName) {
      const removed = tags.value.find(t => t.id === id)
      if (removed?.name === filterQuery.value.tagName) {
        filterQuery.value = { ...filterQuery.value, tagName: null }
        await applyFilter()
      }
    }
  }

  async function setItemTags(itemId: number, tagIds: number[]): Promise<void> {
    await invoke('set_item_tags', { itemId, tagIds })
    await refreshAndFilter()
  }

  // ── Import / Export ─────────────────────────────────────────────────────────

  async function importBibFile(path: string, mode: string): Promise<ImportResult> {
    const result = await invoke<ImportResult>('import_bib_file', { path, mode })
    await refreshAndFilter()
    await loadCollections()
    return result
  }

  async function exportBibFile(itemIds: number[], path: string): Promise<number> {
    return invoke<number>('export_bib_file', { itemIds, path })
  }

  // ── Attachments ─────────────────────────────────────────────────────────────

  async function pickAndAttachFile(itemId: number): Promise<Attachment | null> {
    return invoke<Attachment | null>('pick_and_attach_file', { itemId })
  }

  async function getItemAttachments(itemId: number): Promise<Attachment[]> {
    return invoke<Attachment[]>('get_item_attachments', { itemId })
  }

  async function removeAttachment(id: number): Promise<void> {
    return invoke('remove_attachment', { id })
  }

  async function openAttachmentExternal(id: number): Promise<void> {
    return invoke('open_attachment_external', { id })
  }

  return {
    // state
    items, displayItems, loading, error, collections, tags, filterQuery,
    // items
    loadItems, applyFilter, refreshAndFilter,
    createItem, updateItem, deleteItem, searchItems,
    // collections
    loadCollections, createCollection, renameCollection, deleteCollection,
    addItemToCollection, removeItemFromCollection, getItemCollectionIds,
    // tags
    loadTags, createTag, updateTagColor, deleteTag, setItemTags,
    // import / export
    importBibFile, exportBibFile,
    // attachments
    pickAndAttachFile, getItemAttachments, removeAttachment, openAttachmentExternal,
  }
}
