<script setup lang="ts">
import { ref, watch } from 'vue'

export interface SectionHeading {
  id: string
  text: string
  displayNum: string
  level: number
}

const props = defineProps<{
  items: SectionHeading[]
  command: (item: SectionHeading) => void
}>()

const selectedIndex = ref(0)

watch(
  () => props.items,
  () => { selectedIndex.value = 0 },
)

function select(item: SectionHeading) {
  props.command(item)
}

function onKeyDown(event: KeyboardEvent): boolean {
  if (event.key === 'ArrowUp') {
    selectedIndex.value = (selectedIndex.value - 1 + props.items.length) % props.items.length
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
  <div class="ref-suggest" v-if="items.length">
    <div class="suggest-header">Jump to section</div>
    <button
      v-for="(item, i) in items"
      :key="item.id"
      class="ref-suggest-item"
      :class="{ 'ref-suggest-item--active': i === selectedIndex }"
      :style="{ paddingLeft: `${10 + (item.level - 1) * 14}px` }"
      @mouseenter="selectedIndex = i"
      @mousedown.prevent="select(item)"
    >
      <span class="ref-num">§ {{ item.displayNum }}</span>
      <span class="ref-title">{{ item.text }}</span>
    </button>
  </div>
  <div class="ref-suggest ref-suggest--empty" v-else>
    No headings in this document
  </div>
</template>

<style scoped>
.ref-suggest {
  background: rgba(252, 251, 249, 0.98);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 10px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.14), 0 2px 6px rgba(0, 0, 0, 0.07);
  min-width: 280px;
  max-width: 360px;
  overflow: hidden;
  animation: suggestIn 0.14s cubic-bezier(0.34, 1.2, 0.64, 1);
  transform-origin: top left;
}

@keyframes suggestIn {
  from { opacity: 0; transform: scale(0.96) translateY(-4px); }
  to   { opacity: 1; transform: scale(1) translateY(0); }
}

.ref-suggest--empty {
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

.ref-suggest-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 7px 10px;
  background: none;
  border: none;
  cursor: pointer;
  text-align: left;
  transition: background 0.12s;
}

.ref-suggest-item--active {
  background: rgba(124, 58, 237, 0.07);
}

.ref-num {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  font-size: 10px;
  font-weight: 600;
  color: #7C3AED;
  background: rgba(124, 58, 237, 0.08);
  padding: 2px 6px;
  border-radius: 4px;
  flex-shrink: 0;
  white-space: nowrap;
}

.ref-title {
  font-size: 12.5px;
  font-weight: 500;
  color: #1A1917;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}
</style>
