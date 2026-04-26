<script setup lang="ts">
import { computed } from 'vue'
import { nodeViewProps, NodeViewWrapper } from '@tiptap/vue-3'
import { emitter } from '../events'
import { useDocument } from '../composables/useDocument'

const props = defineProps(nodeViewProps)
const { citations } = useDocument()

const resolved = computed(() =>
  citations.value.some(c => c.key === props.node.attrs.citeKey)
)

function onMouseEnter(e: MouseEvent) {
  const el = e.currentTarget as HTMLElement
  const rect = el.getBoundingClientRect()
  emitter.emit('cite:hover', { key: props.node.attrs.citeKey, rect })
}

function onMouseLeave() {
  emitter.emit('cite:leave')
}

function onClick() {
  emitter.emit('cite:click', { key: props.node.attrs.citeKey })
}
</script>

<template>
  <NodeViewWrapper as="span" class="cite-inline-node">
    <span
      :class="['cite-sup-node', { 'cite-sup-unresolved': !resolved }]"
      :title="!resolved ? `No entry for '${node.attrs.citeKey}' in references.bib` : undefined"
      @mouseenter="onMouseEnter"
      @mouseleave="onMouseLeave"
      @click.stop="onClick"
    >{{ resolved ? `[${node.attrs.displayIndex}]` : '[?]' }}</span>
  </NodeViewWrapper>
</template>
