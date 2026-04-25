<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import TitleBar from './components/TitleBar.vue'
import Sidebar from './components/Sidebar.vue'
import StatusBar from './components/StatusBar.vue'
import { useDocument } from './composables/useDocument'

const route = useRoute()
const { docTitle, isDirty } = useDocument()

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
