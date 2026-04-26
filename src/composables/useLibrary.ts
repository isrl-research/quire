import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

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

export interface SearchQuery {
  text?: string
  entryTypes?: string[]
  yearMin?: string
  yearMax?: string
  hasDoi?: boolean
}

// Module-level singletons — same pattern as useDocument
const items = ref<LibraryItem[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

export function useLibrary() {
  async function loadItems(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      items.value = await invoke<LibraryItem[]>('get_library_items')
    } catch (e) {
      error.value = String(e)
      console.error('[library] Failed to load items:', e)
    } finally {
      loading.value = false
    }
  }

  async function searchItems(query: SearchQuery): Promise<LibraryItem[]> {
    try {
      return await invoke<LibraryItem[]>('search_library_items', { query })
    } catch (e) {
      console.error('[library] Search failed:', e)
      return []
    }
  }

  async function createItem(input: ItemInput): Promise<LibraryItem> {
    const created = await invoke<LibraryItem>('create_library_item', { item: input })
    items.value.unshift(created)
    return created
  }

  async function updateItem(id: number, input: ItemInput): Promise<LibraryItem> {
    const updated = await invoke<LibraryItem>('update_library_item', { id, item: input })
    const idx = items.value.findIndex(i => i.id === id)
    if (idx !== -1) items.value[idx] = updated
    return updated
  }

  async function deleteItem(id: number): Promise<void> {
    await invoke('delete_library_item', { id })
    items.value = items.value.filter(i => i.id !== id)
  }

  return { items, loading, error, loadItems, searchItems, createItem, updateItem, deleteItem }
}
