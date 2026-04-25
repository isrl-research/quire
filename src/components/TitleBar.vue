<script setup lang="ts">
interface Props {
  title: string
  subtitle: string
  exportLabel?: string | null
  isDirty?: boolean
  onExport?: () => void
}
defineProps<Props>()
const emit = defineEmits<{ toggleMenu: [] }>()
</script>

<template>
  <header class="titlebar">
    <div class="traffic-lights">
      <span class="dot red"></span>
      <span class="dot yellow"></span>
      <span class="dot green"></span>
      <button class="hamburger" @click="emit('toggleMenu')" title="Menu">
        <svg width="13" height="11" viewBox="0 0 13 11" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
          <line x1="0" y1="1.5" x2="13" y2="1.5"/>
          <line x1="0" y1="5.5" x2="13" y2="5.5"/>
          <line x1="0" y1="9.5" x2="13" y2="9.5"/>
        </svg>
      </button>
    </div>

    <div class="title-center">
      <span v-if="isDirty" class="dirty-dot" title="Unsaved changes">●</span>
      <span class="doc-title">{{ title }}</span>
      <span v-if="subtitle" class="sep">—</span>
      <span v-if="subtitle" class="doc-subtitle">{{ subtitle }}</span>
    </div>

    <div class="titlebar-end">
      <button v-if="exportLabel" class="export-btn" @click="onExport?.()">
        {{ exportLabel }}
      </button>
    </div>
  </header>
</template>

<style scoped>
.titlebar {
  height: var(--titlebar-h);
  background: var(--bg-chrome);
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  padding: 0 14px;
  flex-shrink: 0;
  user-select: none;
  -webkit-user-select: none;
  position: relative;
  z-index: 10;
}

.traffic-lights {
  display: flex;
  gap: 6px;
  align-items: center;
  width: 88px;
  flex-shrink: 0;
}

.hamburger {
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
  margin-left: 2px;
}
.hamburger:hover {
  background: var(--bg-chrome-active);
  color: var(--text-secondary);
}

.dot {
  width: 11px;
  height: 11px;
  border-radius: 50%;
  display: block;
}
.dot.red    { background: #FF5F57; box-shadow: 0 0 0 0.5px rgba(0,0,0,0.12) inset; }
.dot.yellow { background: #FFBD2E; box-shadow: 0 0 0 0.5px rgba(0,0,0,0.12) inset; }
.dot.green  { background: #28C840; box-shadow: 0 0 0 0.5px rgba(0,0,0,0.12) inset; }

.title-center {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 7px;
  font-size: 12.5px;
  font-weight: 500;
  letter-spacing: -0.01em;
  overflow: hidden;
}

.dirty-dot {
  font-size: 10px;
  color: var(--text-tertiary);
  line-height: 1;
  flex-shrink: 0;
}

.doc-title {
  font-weight: 600;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sep {
  color: var(--text-tertiary);
  font-weight: 300;
  flex-shrink: 0;
}

.doc-subtitle {
  color: var(--text-secondary);
  font-weight: 400;
  white-space: nowrap;
}

.titlebar-end {
  width: 130px;
  display: flex;
  justify-content: flex-end;
  flex-shrink: 0;
}

.export-btn {
  background: var(--bg-chrome-active);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-pill);
  padding: 4px 11px;
  font-size: 11px;
  font-weight: 500;
  color: var(--text);
  cursor: pointer;
  transition: background var(--t), border-color var(--t), box-shadow var(--t);
  white-space: nowrap;
  letter-spacing: -0.01em;
}
.export-btn:hover {
  background: rgba(0,0,0,0.07);
  border-color: rgba(0,0,0,0.18);
  box-shadow: var(--shadow-xs);
}
.export-btn:active {
  background: rgba(0,0,0,0.1);
}
</style>
