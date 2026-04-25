import { invoke } from '@tauri-apps/api/core'
import { useDocument } from './useDocument'

export interface RecentFile {
  path: string
  title: string
  last_opened: string // unix seconds as string
}

export function useQuire() {
  const { citations, bibPath } = useDocument()

  async function initQuire(): Promise<void> {
    try {
      const dir = await invoke<string>('get_quire_dir')
      bibPath.value = `${dir}/references.bib`
      const entries = await invoke<typeof citations.value>('get_global_bib')
      citations.value = entries
    } catch (e) {
      console.error('[quire] Failed to init:', e)
    }
  }

  async function getRecentFiles(): Promise<RecentFile[]> {
    try {
      return await invoke<RecentFile[]>('get_recent_files')
    } catch {
      return []
    }
  }

  async function addRecentFile(path: string, title: string): Promise<void> {
    try {
      await invoke('add_recent_file', { path, title })
    } catch (e) {
      console.warn('[quire] Could not update recent files:', e)
    }
  }

  /** Human-readable relative time from a unix-seconds string */
  function relativeTime(unixSecsStr: string): string {
    const secs = parseInt(unixSecsStr, 10)
    if (isNaN(secs)) return ''
    const diff = Math.floor(Date.now() / 1000) - secs
    if (diff < 60) return 'Just now'
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`
    if (diff < 172800) return 'Yesterday'
    return `${Math.floor(diff / 86400)}d ago`
  }

  return { initQuire, getRecentFiles, addRecentFile, relativeTime }
}
