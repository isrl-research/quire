import { Extension } from '@tiptap/core'
import Suggestion from '@tiptap/suggestion'
import { PluginKey } from '@tiptap/pm/state'
import { VueRenderer } from '@tiptap/vue-3'
import SectionRefSuggestList, { type SectionHeading } from '../components/SectionRefSuggestList.vue'
import { computeHeadingMap } from './SectionRef'

const sectionRefSuggestKey = new PluginKey('sectionRefSuggest')

export const SectionRefSuggest = Extension.create({
  name: 'sectionRefSuggest',

  addOptions() {
    return {
      suggestion: {
        char: '[[',
        pluginKey: sectionRefSuggestKey,
        allowSpaces: true,

        items({ query, editor }: { query: string; editor: any }): SectionHeading[] {
          const doc = editor.state.doc
          const headingMap = computeHeadingMap(doc)
          const results: SectionHeading[] = []
          doc.descendants((node: any) => {
            if (node.type.name !== 'heading') return
            const id = node.attrs.id as string | null
            if (!id) return
            const displayNum = headingMap.get(id) ?? '?'
            results.push({ id, text: node.textContent as string, displayNum, level: node.attrs.level })
          })
          if (!query) return results.slice(0, 10)
          const q = query.toLowerCase()
          return results.filter(h => h.text.toLowerCase().includes(q)).slice(0, 10)
        },

        render() {
          let renderer: VueRenderer
          let wrapper: HTMLDivElement

          return {
            onStart(props: any) {
              wrapper = document.createElement('div')
              wrapper.style.cssText = 'position:fixed;z-index:9999;'
              document.body.appendChild(wrapper)

              renderer = new VueRenderer(SectionRefSuggestList, {
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
          props: item,
        }: {
          editor: any
          range: any
          props: SectionHeading
        }) {
          editor
            .chain()
            .focus()
            .deleteRange(range)
            .insertContent({
              type: 'sectionRef',
              attrs: { headingId: item.id, displayNum: item.displayNum },
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
