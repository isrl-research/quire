<script setup lang="ts">
import { computed } from 'vue'
import { nodeViewProps, NodeViewWrapper } from '@tiptap/vue-3'

const props = defineProps(nodeViewProps)

// If the plugin set displayNum to '?' the heading was deleted
const resolved = computed(() => props.node.attrs.displayNum !== '?')

function onClick() {
  const headingId = props.node.attrs.headingId as string
  let headingPos: number | null = null
  props.editor.state.doc.descendants((node: any, pos: number) => {
    if (node.type.name === 'heading' && node.attrs.id === headingId) {
      headingPos = pos
      return false
    }
  })
  if (headingPos !== null) {
    props.editor.chain().setTextSelection(headingPos + 1).scrollIntoView().run()
  }
}
</script>

<template>
  <NodeViewWrapper as="span" class="section-ref-inline">
    <span
      :class="['section-ref-node', { 'section-ref-unresolved': !resolved }]"
      :title="resolved ? `Jump to § ${node.attrs.displayNum}` : 'Section not found'"
      @click.stop="onClick"
    >§ {{ node.attrs.displayNum }}</span>
  </NodeViewWrapper>
</template>
