import { Node, mergeAttributes } from '@tiptap/core'
import { Plugin, PluginKey } from '@tiptap/pm/state'
import { VueNodeViewRenderer } from '@tiptap/vue-3'
import SectionRefNodeView from '../components/SectionRefNodeView.vue'

export const sectionRefKey = new PluginKey('sectionRef')

// Mirrors the CSS counter logic in editor.css exactly.
export function computeHeadingMap(doc: any): Map<string, string> {
  const map = new Map<string, string>()
  let s = 0
  let ss = 0
  let sss = 0
  doc.descendants((node: any) => {
    if (node.type.name !== 'heading') return
    const id = node.attrs.id as string | null
    const level = node.attrs.level as number
    if (level === 1) { s++; ss = 0; sss = 0; if (id) map.set(id, `${s}`) }
    else if (level === 2) { ss++; sss = 0; if (id) map.set(id, `${s}.${ss}`) }
    else if (level === 3) { sss++; if (id) map.set(id, `${s}.${ss}.${sss}`) }
  })
  return map
}

export const SectionRef = Node.create({
  name: 'sectionRef',
  group: 'inline',
  inline: true,
  atom: true,

  addAttributes() {
    return {
      headingId: {
        default: null,
        parseHTML: (el: HTMLElement) => el.getAttribute('data-ref-id'),
        renderHTML: (attrs: any) => ({ 'data-ref-id': attrs.headingId }),
      },
      displayNum: {
        default: '?',
        parseHTML: (el: HTMLElement) => el.getAttribute('data-ref-num') ?? '?',
        renderHTML: (attrs: any) => ({ 'data-ref-num': attrs.displayNum }),
      },
    }
  },

  parseHTML() {
    return [{ tag: 'span[data-ref-id]' }]
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'span',
      mergeAttributes(HTMLAttributes, { class: 'section-ref-rendered' }),
      `§ ${HTMLAttributes['data-ref-num']}`,
    ]
  },

  addNodeView() {
    return VueNodeViewRenderer(SectionRefNodeView)
  },

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: sectionRefKey,
        appendTransaction(transactions, _oldState, newState) {
          // Prevent re-entry: skip if any transaction in this batch is ours
          if (transactions.some(tr => tr.getMeta('headingRefSync'))) return null

          const docChanged = transactions.some(tr => tr.docChanged)
          if (!docChanged) return null

          const tr = newState.tr
          let changed = false

          // Job 1: assign stable UUIDs to any heading that lacks one
          newState.doc.descendants((node: any, pos: number) => {
            if (node.type.name === 'heading' && !node.attrs.id) {
              tr.setNodeMarkup(pos, undefined, { ...node.attrs, id: crypto.randomUUID() })
              changed = true
            }
          })

          // Job 2: sync sectionRef displayNum using tr.doc (includes Job 1 IDs)
          const headingMap = computeHeadingMap(tr.doc)
          tr.doc.descendants((node: any, pos: number) => {
            if (node.type.name !== 'sectionRef') return
            const newNum = headingMap.get(node.attrs.headingId) ?? '?'
            if (node.attrs.displayNum !== newNum) {
              tr.setNodeMarkup(pos, undefined, { ...node.attrs, displayNum: newNum })
              changed = true
            }
          })

          if (!changed) return null
          tr.setMeta('headingRefSync', true)
          return tr
        },
      }),
    ]
  },
})
