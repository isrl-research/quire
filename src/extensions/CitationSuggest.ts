import { Extension } from '@tiptap/core'
import Suggestion from '@tiptap/suggestion'
import { VueRenderer } from '@tiptap/vue-3'
import CitationSuggestList from '../components/CitationSuggestList.vue'
import { useDocument, type BibEntry } from '../composables/useDocument'

export const CitationSuggest = Extension.create({
  name: 'citationSuggest',

  addOptions() {
    return {
      suggestion: {
        char: '@',
        allowSpaces: false,

        items({ query }: { query: string }): BibEntry[] {
          const { citations } = useDocument()
          const q = query.toLowerCase()
          const all = citations.value
          if (!q) return all.slice(0, 8)
          return all
            .filter(
              c =>
                c.key.toLowerCase().includes(q) ||
                (c.title ?? '').toLowerCase().includes(q) ||
                (c.authors ?? '').toLowerCase().includes(q),
            )
            .slice(0, 8)
        },

        render() {
          let renderer: VueRenderer
          let wrapper: HTMLDivElement

          return {
            onStart(props: any) {
              wrapper = document.createElement('div')
              wrapper.style.cssText = 'position:fixed;z-index:9999;'
              document.body.appendChild(wrapper)

              renderer = new VueRenderer(CitationSuggestList, {
                props,
                editor: props.editor,
              })
              wrapper.appendChild(renderer.el!)

              const rect: DOMRect | undefined = props.clientRect?.()
              if (rect) {
                wrapper.style.left = `${rect.left}px`
                wrapper.style.top = `${rect.bottom + 4}px`
              }
            },

            onUpdate(props: any) {
              renderer.updateProps(props)

              const rect: DOMRect | undefined = props.clientRect?.()
              if (rect) {
                wrapper.style.left = `${rect.left}px`
                wrapper.style.top = `${rect.bottom + 4}px`
              }
            },

            onKeyDown({ event }: { event: KeyboardEvent }): boolean {
              if (event.key === 'Escape') {
                wrapper.style.display = 'none'
                return true
              }
              return (renderer.ref as any)?.onKeyDown(event) ?? false
            },

            onExit() {
              renderer.destroy()
              wrapper.remove()
            },
          }
        },

        command({
          editor,
          range,
          props: entry,
        }: {
          editor: any
          range: any
          props: BibEntry
        }) {
          // Compute next sequential index, or reuse existing one for this key
          const usedKeys = new Map<string, number>()
          let maxIndex = 0
          editor.state.doc.descendants((node: any) => {
            if (node.type.name === 'citation') {
              const k = node.attrs.citeKey as string
              const idx = node.attrs.displayIndex as number
              if (!usedKeys.has(k)) {
                usedKeys.set(k, idx)
                maxIndex = Math.max(maxIndex, idx)
              }
            }
          })
          const displayIndex = usedKeys.has(entry.key)
            ? usedKeys.get(entry.key)!
            : maxIndex + 1

          editor
            .chain()
            .focus()
            .deleteRange(range)
            .insertContent({
              type: 'citation',
              attrs: { citeKey: entry.key, displayIndex },
            })
            .run()
        },
      },
    }
  },

  addProseMirrorPlugins() {
    return [
      Suggestion({
        editor: this.editor,
        ...this.options.suggestion,
      }),
    ]
  },
})
