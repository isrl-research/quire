<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import TitleBar from './components/TitleBar.vue'
import Sidebar from './components/Sidebar.vue'
import StatusBar from './components/StatusBar.vue'
import HamburgerMenu from './components/HamburgerMenu.vue'
import { useDocument } from './composables/useDocument'
import { useQuire } from './composables/useQuire'
import { useFileOps } from './composables/useFileOps'
import { emitter } from './events'

const route = useRoute()
const router = useRouter()
const { docTitle, filePath, isDirty } = useDocument()
const { initQuire, addRecentFile } = useQuire()
useFileOps()

// ── Menu ──────────────────────────────────────────────────────────────────────

const menuOpen = ref(false)

function toggleMenu() {
  menuOpen.value = !menuOpen.value
}

function handleNewDoc() {
  // Reset document state — WriteView listens for doc:opened with empty content
  const doc = useDocument()
  doc.docTitle.value = 'Untitled Document'
  doc.docAuthors.value = [{ name: '[Author]', orcid: '', title: '', affiliation: '' }]
  doc.filePath.value = null
  doc.isDirty.value = false
  emitter.emit('doc:opened', {
    path: '',
    content: `<h2>Abstract</h2><p></p><h2>1. Introduction</h2><p></p>`,
  })
  router.push('/write')
}

// ── Title config ──────────────────────────────────────────────────────────────

const titleConfig = computed(() => {
  switch (route.name) {
    case 'write':
      return { title: docTitle.value, subtitle: 'Write', exportLabel: 'Export as PDF' }
    case 'workbench':
      return { title: 'Workbench', subtitle: docTitle.value, exportLabel: null }
    case 'pdf':
      return { title: 'Popova et al. 2022', subtitle: 'PDF Viewer', exportLabel: 'Export as .bib' }
    default:
      return { title: 'Quire', subtitle: '', exportLabel: null }
  }
})

// ── Export ────────────────────────────────────────────────────────────────────

async function handleExport() {
  if (!filePath.value) return
  emitter.emit('export:start')
  try {
    const result = await invoke<string>('run_quarto', {
      docPath: filePath.value,
      format: 'pdf',
    })
    const outputPath = result.split('Output: ')[1]?.trim() ?? ''
    emitter.emit('export:done', { outputPath })
  } catch (e) {
    emitter.emit('export:error', { message: String(e) })
  }
}

// ── Keyboard shortcuts (app-level) ────────────────────────────────────────────

function onKeydown(e: KeyboardEvent) {
  const ctrl = e.ctrlKey || e.metaKey
  if (ctrl && e.key === 'n') {
    e.preventDefault()
    handleNewDoc()
  }
  if (e.key === 'Escape' && menuOpen.value) {
    menuOpen.value = false
  }
}

// ── Recent files tracking ─────────────────────────────────────────────────────

emitter.on('doc:saved', ({ path }) => {
  addRecentFile(path, docTitle.value)
})
emitter.on('doc:opened', ({ path }) => {
  if (path) addRecentFile(path, docTitle.value)
})

// ── Lifecycle ─────────────────────────────────────────────────────────────────

onMounted(async () => {
  await initQuire()
  window.addEventListener('keydown', onKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
  emitter.off('doc:saved')
  emitter.off('doc:opened')
})
</script>

<template>
  <div class="app-shell">
    <TitleBar
      :title="titleConfig.title"
      :subtitle="titleConfig.subtitle"
      :export-label="titleConfig.exportLabel"
      :is-dirty="isDirty"
      :on-export="handleExport"
      @toggle-menu="toggleMenu"
    />

    <HamburgerMenu
      v-if="menuOpen"
      @close="menuOpen = false"
      @new-doc="handleNewDoc"
    />

    <div class="app-body">
      <Sidebar />
      <main class="main-content">
        <RouterView />
      </main>
    </div>
    <StatusBar />
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.app-body {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-height: 0;
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}
</style>
