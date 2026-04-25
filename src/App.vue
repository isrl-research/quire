<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import TitleBar from './components/TitleBar.vue'
import Sidebar from './components/Sidebar.vue'
import StatusBar from './components/StatusBar.vue'
import { useDocument } from './composables/useDocument'
import { emitter } from './events'

const route = useRoute()
const { docTitle, filePath, isDirty } = useDocument()

async function handleExport() {
  if (!filePath.value) return
  const format = route.name === 'pdf' ? 'pdf' : 'pdf'
  emitter.emit('export:start')
  try {
    const result = await invoke<string>('run_quarto', { docPath: filePath.value, format })
    const outputPath = result.split('Output: ')[1]?.trim() ?? ''
    emitter.emit('export:done', { outputPath })
  } catch (e) {
    emitter.emit('export:error', { message: String(e) })
  }
}

const titleConfig = computed(() => {
  switch (route.name) {
    case 'write':
      return {
        title: docTitle.value,
        subtitle: 'Write',
        exportLabel: 'Export as PDF',
      }
    case 'workbench':
      return {
        title: 'Workbench',
        subtitle: docTitle.value,
        exportLabel: null,
      }
    case 'pdf':
      return {
        title: 'Popova et al. 2022',
        subtitle: 'PDF Viewer',
        exportLabel: 'Export as .bib',
      }
    default:
      return { title: 'Quire', subtitle: '', exportLabel: null }
  }
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
