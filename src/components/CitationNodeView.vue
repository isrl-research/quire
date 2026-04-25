<script setup lang="ts">
import { nodeViewProps, NodeViewWrapper } from '@tiptap/vue-3'
import { emitter } from '../events'

const props = defineProps(nodeViewProps)

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
    <sup
      class="cite-sup-node"
      @mouseenter="onMouseEnter"
      @mouseleave="onMouseLeave"
      @click.stop="onClick"
    >[{{ node.attrs.displayIndex }}]</sup>
  </NodeViewWrapper>
</template>
