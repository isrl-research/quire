import { invoke } from '@tauri-apps/api/core'
import { useDocument, type BibEntry } from './useDocument'
import { emitter } from '../events'

interface OpenResult {
  path: string
  content: string
  frontmatter: { title?: string; authors?: string[]; date?: string }
  body: string
}

export function useFileOps() {
  const { filePath, bibPath, docTitle, docAuthors, isDirty, lastSaved, citations } = useDocument()

  async function openDocument(): Promise<string | null> {
    try {
      const result = await invoke<OpenResult>('open_document')
      filePath.value = result.path
      if (result.frontmatter.title) docTitle.value = result.frontmatter.title
      if (result.frontmatter.authors?.length) {
        docAuthors.value = result.frontmatter.authors.map(a =>
          typeof a === 'string'
            ? { name: a, orcid: '', title: '', affiliation: '' }
            : a
        )
      }
      isDirty.value = false
      emitter.emit('doc:opened', { path: result.path, content: result.body })

      // Auto-detect and load .bib
      tryLoadBib(result.path)
      return result.body
    } catch (e) {
      // User cancelled or error
      return null
    }
  }

  async function saveDocument(content: string): Promise<boolean> {
    if (!filePath.value) {
      return saveDocumentAs(content)
    }
    try {
      await invoke('save_document', { path: filePath.value, content })
      isDirty.value = false
      lastSaved.value = new Date()
      emitter.emit('doc:saved', { path: filePath.value })
      return true
    } catch (e) {
      console.error('Save failed:', e)
      return false
    }
  }

  async function saveDocumentAs(content: string): Promise<boolean> {
    try {
      const newPath = await invoke<string>('save_document_as', { content })
      filePath.value = newPath
      isDirty.value = false
      lastSaved.value = new Date()
      emitter.emit('doc:saved', { path: newPath })
      return true
    } catch {
      return false
    }
  }

  async function tryLoadBib(docPath: string) {
    try {
      const bib = await invoke<string | null>('find_bib_for_document', { docPath })
      if (!bib) return
      bibPath.value = bib
      const entries = await invoke<BibEntry[]>('load_bib', { path: bib })
      citations.value = entries
      emitter.emit('bib:loaded')
    } catch (e) {
      console.warn('Could not load .bib:', e)
    }
  }

  async function loadBibManual(path: string) {
    try {
      const entries = await invoke<BibEntry[]>('load_bib', { path })
      bibPath.value = path
      citations.value = entries
      emitter.emit('bib:loaded')
    } catch (e) {
      console.error('Bib load failed:', e)
    }
  }

  return { openDocument, saveDocument, saveDocumentAs, loadBibManual }
}
