<script setup lang="ts">
import { ref, watch } from 'vue'
import type { BibEntry } from '../composables/useDocument'

const props = defineProps<{
  items: BibEntry[]
  command: (item: BibEntry) => void
}>()

const selectedIndex = ref(0)

watch(
  () => props.items,
  () => { selectedIndex.value = 0 },
)

function select(item: BibEntry) {
  props.command(item)
}

function onKeyDown(event: KeyboardEvent): boolean {
  if (event.key === 'ArrowUp') {
    selectedIndex.value =
      (selectedIndex.value - 1 + props.items.length) % props.items.length
    return true
  }
  if (event.key === 'ArrowDown') {
    selectedIndex.value = (selectedIndex.value + 1) % props.items.length
    return true
  }
  if (event.key === 'Enter') {
    const item = props.items[selectedIndex.value]
    if (item) select(item)
    return true
  }
  return false
}

defineExpose({ onKeyDown })
</script>

<template>
  <div class="cite-suggest" v-if="items.length">
    <div class="suggest-header">Cite from bibliography</div>
    <button
      v-for="(item, i) in items"
      :key="item.key"
      class="suggest-item"
      :class="{ 'suggest-item--active': i === selectedIndex }"
      @mouseenter="selectedIndex = i"
      @mousedown.prevent="select(item)"
    >
      <span class="suggest-key">{{ item.key }}</span>
      <span class="suggest-body">
        <span class="suggest-title">{{ item.title }}</span>
        <span class="suggest-meta">{{ item.authors }}{{ item.year ? ' · ' + item.year : '' }}</span>
      </span>
    </button>
  </div>
  <div class="cite-suggest cite-suggest--empty" v-else>
    No matches in references.bib
  </div>
</template>

<style scoped>
.cite-suggest {
  background: rgba(252, 251, 249, 0.98);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 10px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.14), 0 2px 6px rgba(0, 0, 0, 0.07);
  min-width: 340px;
  max-width: 400px;
  overflow: hidden;
  animation: suggestIn 0.14s cubic-bezier(0.34, 1.2, 0.64, 1);
  transform-origin: top left;
}

@keyframes suggestIn {
  from { opacity: 0; transform: scale(0.96) translateY(-4px); }
  to   { opacity: 1; transform: scale(1) translateY(0); }
}

.cite-suggest--empty {
  padding: 10px 14px;
  font-size: 12px;
  color: #8E8E93;
  font-style: italic;
}

.suggest-header {
  padding: 7px 12px 5px;
  font-size: 9.5px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  color: #8E8E93;
  border-bottom: 1px solid rgba(0, 0, 0, 0.07);
}

.suggest-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 8px 10px;
  background: none;
  border: none;
  cursor: pointer;
  text-align: left;
  transition: background 0.12s;
}

.suggest-item--active {
  background: rgba(10, 95, 191, 0.08);
}

.suggest-key {
  font-family: ui-monospace, "SF Mono", Menlo, monospace;
  font-size: 10px;
  color: #0A5FBF;
  background: rgba(10, 95, 191, 0.08);
  padding: 2px 6px;
  border-radius: 4px;
  flex-shrink: 0;
  white-space: nowrap;
}

.suggest-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1px;
  overflow: hidden;
}

.suggest-title {
  font-size: 12.5px;
  font-weight: 500;
  color: #1A1917;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

.suggest-meta {
  font-size: 10.5px;
  color: #8E8E93;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}
</style>
