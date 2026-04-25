import { Node, mergeAttributes } from '@tiptap/core'
import { VueNodeViewRenderer } from '@tiptap/vue-3'
import CitationNodeView from '../components/CitationNodeView.vue'

export const CitationNode = Node.create({
  name: 'citation',
  group: 'inline',
  inline: true,
  atom: true,

  addAttributes() {
    return {
      citeKey: {
        default: null,
        parseHTML: el => el.getAttribute('data-cite-key'),
        renderHTML: attrs => ({ 'data-cite-key': attrs.citeKey }),
      },
      displayIndex: {
        default: 1,
        parseHTML: el => parseInt(el.getAttribute('data-index') ?? '1', 10),
        renderHTML: attrs => ({ 'data-index': String(attrs.displayIndex) }),
      },
    }
  },

  parseHTML() {
    return [{ tag: 'span[data-cite-key]' }]
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'span',
      mergeAttributes(HTMLAttributes, { class: 'cite-node-rendered' }),
      `[${HTMLAttributes['data-index']}]`,
    ]
  },

  addNodeView() {
    return VueNodeViewRenderer(CitationNodeView)
  },
})
