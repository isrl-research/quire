<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { useQuire, type RecentFile } from '../composables/useQuire'
import { useFileOps } from '../composables/useFileOps'

const emit = defineEmits<{
  close: []
  newDoc: []
}>()

const router = useRouter()
const { getRecentFiles, relativeTime } = useQuire()
const { openDocument } = useFileOps()

const recentFiles = ref<RecentFile[]>([])

onMounted(async () => {
  recentFiles.value = await getRecentFiles()
  window.addEventListener('keydown', onKey)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKey)
})

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close')
}

function shortPath(fullPath: string): string {
  const home = fullPath.replace(/\\/g, '/').replace(/^C:\/Users\/[^/]+/, '~')
  // Keep last 2 segments for readability
  const parts = home.split('/')
  if (parts.length > 3) return '…/' + parts.slice(-2).join('/')
  return home
}

async function handleNew() {
  emit('newDoc')
  emit('close')
}

async function handleOpen() {
  emit('close')
  await openDocument()
}

async function handleRecent(file: RecentFile) {
  emit('close')
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const { useDocument } = await import('../composables/useDocument')
    const doc = useDocument()
    const result = await invoke<{ path: string; body: string; frontmatter: { title?: string; authors?: string[] } }>(
      'open_document_path',
      { path: file.path }
    )
    if (result.frontmatter.title) doc.docTitle.value = result.frontmatter.title
    if (result.frontmatter.authors?.length) {
      doc.docAuthors.value = result.frontmatter.authors.map((a: any) =>
        typeof a === 'string' ? { name: a, orcid: '', title: '', affiliation: '' } : a
      )
    }
    doc.filePath.value = result.path
    doc.isDirty.value = false
    // Emit the content via events so WriteView can pick it up
    const { emitter } = await import('../events')
    emitter.emit('doc:opened', { path: result.path, content: result.body })
  } catch (e) {
    console.error('Failed to open recent file:', e)
  }
  router.push('/write')
}
</script>

<template>
  <!-- Backdrop -->
  <div class="menu-backdrop" @click="emit('close')"></div>

  <!-- Panel -->
  <div class="menu-panel">
    <div class="menu-header">
      <span class="menu-logo">Quire</span>
      <button class="menu-close" @click="emit('close')">
        <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round">
          <line x1="1" y1="1" x2="12" y2="12"/>
          <line x1="12" y1="1" x2="1" y2="12"/>
        </svg>
      </button>
    </div>

    <!-- Actions -->
    <div class="menu-actions">
      <button class="action-btn" @click="handleNew">
        <span class="action-icon">
          <svg width="15" height="15" viewBox="0 0 15 15" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <rect x="2" y="2" width="11" height="11" rx="2"/>
            <line x1="7.5" y1="5" x2="7.5" y2="10"/>
            <line x1="5" y1="7.5" x2="10" y2="7.5"/>
          </svg>
        </span>
        <span class="action-label">New Document</span>
        <span class="action-hint">Ctrl N</span>
      </button>
      <button class="action-btn" @click="handleOpen">
        <span class="action-icon">
          <svg width="15" height="15" viewBox="0 0 15 15" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M2 5h4l1.5-2H13v9H2V5Z"/>
          </svg>
        </span>
        <span class="action-label">Open File…</span>
        <span class="action-hint">Ctrl O</span>
      </button>
    </div>

    <!-- Recent files -->
    <div class="recent-section">
      <div class="recent-label">Recent</div>
      <div class="recent-list" v-if="recentFiles.length">
        <button
          v-for="file in recentFiles"
          :key="file.path"
          class="recent-item"
          @click="handleRecent(file)"
        >
          <span class="recent-icon">
            <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
              <path d="M8 1H3a1 1 0 00-1 1v9a1 1 0 001 1h7a1 1 0 001-1V5L8 1Z"/>
              <path d="M8 1v4h4"/>
            </svg>
          </span>
          <span class="recent-body">
            <span class="recent-title">{{ file.title }}</span>
            <span class="recent-path">{{ shortPath(file.path) }}</span>
          </span>
          <span class="recent-time">{{ relativeTime(file.last_opened) }}</span>
        </button>
      </div>
      <div class="recent-empty" v-else>
        No recent files yet
      </div>
    </div>
  </div>
</template>

<style scoped>
.menu-backdrop {
  position: fixed;
  inset: 0;
  z-index: 500;
  background: transparent;
}

.menu-panel {
  position: fixed;
  top: var(--titlebar-h);
  left: calc(var(--sidebar-w) + 8px);
  z-index: 501;
  width: 312px;
  background: rgba(250, 249, 247, 0.97);
  backdrop-filter: blur(32px);
  -webkit-backdrop-filter: blur(32px);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  animation: menuIn 0.18s cubic-bezier(0.34, 1.2, 0.64, 1);
  transform-origin: top left;
}

@keyframes menuIn {
  from { opacity: 0; transform: scale(0.94) translateY(-6px); }
  to   { opacity: 1; transform: scale(1) translateY(0); }
}

.menu-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px 10px;
  border-bottom: 1px solid var(--border);
}

.menu-logo {
  font-size: 12px;
  font-weight: 700;
  color: var(--text);
  letter-spacing: -0.02em;
}

.menu-close {
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
.menu-close:hover {
  background: var(--bg-chrome-active);
  color: var(--text);
}

/* Actions */
.menu-actions {
  padding: 8px;
  border-bottom: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 8px 10px;
  background: none;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  text-align: left;
  transition: background var(--t);
}
.action-btn:hover {
  background: var(--bg-chrome-active);
}

.action-icon {
  width: 26px;
  height: 26px;
  background: var(--surface-solid);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  flex-shrink: 0;
  box-shadow: var(--shadow-xs);
}

.action-label {
  flex: 1;
  font-size: 13px;
  font-weight: 500;
  color: var(--text);
}

.action-hint {
  font-size: 10.5px;
  color: var(--text-tertiary);
  background: var(--bg-chrome-active);
  padding: 2px 6px;
  border-radius: var(--radius-xs);
  font-variant-numeric: tabular-nums;
}

/* Recent */
.recent-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 8px 0 4px;
}

.recent-label {
  font-size: 9.5px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--text-tertiary);
  padding: 0 14px 6px;
}

.recent-list {
  overflow-y: auto;
  flex: 1;
  padding: 0 8px 8px;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.recent-item {
  display: flex;
  align-items: center;
  gap: 9px;
  width: 100%;
  padding: 8px 8px;
  background: none;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  text-align: left;
  transition: background var(--t);
}
.recent-item:hover {
  background: var(--bg-chrome-active);
}

.recent-icon {
  color: var(--text-tertiary);
  flex-shrink: 0;
  display: flex;
  align-items: center;
}

.recent-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
}

.recent-title {
  font-size: 12.5px;
  font-weight: 500;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.recent-path {
  font-size: 10.5px;
  color: var(--text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: var(--font-mono);
}

.recent-time {
  font-size: 10.5px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.recent-empty {
  padding: 12px 14px;
  font-size: 12px;
  color: var(--text-tertiary);
  font-style: italic;
}
</style>
