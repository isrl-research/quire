import { Extension } from '@tiptap/core'
import { Plugin, PluginKey, TextSelection } from '@tiptap/pm/state'
import { Decoration, DecorationSet } from '@tiptap/pm/view'

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    searchAndReplace: {
      setSearch: (term: string) => ReturnType
      findNext: () => ReturnType
      findPrev: () => ReturnType
      replaceOne: (replacement: string) => ReturnType
      replaceAll: (replacement: string) => ReturnType
    }
  }
}

export interface SearchState {
  term: string
  matches: { from: number; to: number }[]
  current: number
}

export const searchKey = new PluginKey<SearchState>('searchReplace')

function findMatches(doc: any, term: string): { from: number; to: number }[] {
  if (!term) return []
  const results: { from: number; to: number }[] = []
  const lower = term.toLowerCase()
  doc.descendants((node: any, pos: number) => {
    if (!node.isText || !node.text) return
    const text = (node.text as string).toLowerCase()
    let i = text.indexOf(lower)
    while (i !== -1) {
      results.push({ from: pos + i, to: pos + i + term.length })
      i = text.indexOf(lower, i + 1)
    }
  })
  return results
}

export const SearchAndReplace = Extension.create({
  name: 'searchAndReplace',

  addCommands() {
    return {
      setSearch:
        (term: string) =>
        ({ tr, state, dispatch }: any) => {
          const matches = findMatches(state.doc, term)
          if (dispatch) dispatch(tr.setMeta(searchKey, { term, matches, current: -1 }))
          return true
        },

      findNext:
        () =>
        ({ tr, state, dispatch }: any) => {
          const ps: SearchState = searchKey.getState(state)!
          if (!ps.matches.length) return false
          const next = (ps.current + 1) % ps.matches.length
          const m = ps.matches[next]
          if (dispatch) {
            dispatch(
              tr
                .setMeta(searchKey, { ...ps, current: next })
                .setSelection(TextSelection.create(tr.doc, m.from, m.to))
                .scrollIntoView()
            )
          }
          return true
        },

      findPrev:
        () =>
        ({ tr, state, dispatch }: any) => {
          const ps: SearchState = searchKey.getState(state)!
          if (!ps.matches.length) return false
          const prev = (ps.current - 1 + ps.matches.length) % ps.matches.length
          const m = ps.matches[prev]
          if (dispatch) {
            dispatch(
              tr
                .setMeta(searchKey, { ...ps, current: prev })
                .setSelection(TextSelection.create(tr.doc, m.from, m.to))
                .scrollIntoView()
            )
          }
          return true
        },

      replaceOne:
        (replacement: string) =>
        ({ tr, state, dispatch }: any) => {
          const ps: SearchState = searchKey.getState(state)!
          if (!ps.matches.length || ps.current < 0) return false
          const { from, to } = ps.matches[ps.current]
          tr.insertText(replacement, from, to)
          const newMatches = findMatches(tr.doc, ps.term)
          const newCurrent = Math.min(ps.current, newMatches.length - 1)
          if (dispatch) dispatch(tr.setMeta(searchKey, { term: ps.term, matches: newMatches, current: newCurrent }))
          return true
        },

      replaceAll:
        (replacement: string) =>
        ({ tr, state, dispatch }: any) => {
          const ps: SearchState = searchKey.getState(state)!
          if (!ps.matches.length) return false
          for (let i = ps.matches.length - 1; i >= 0; i--) {
            const from = tr.mapping.map(ps.matches[i].from)
            const to = tr.mapping.map(ps.matches[i].to)
            tr.insertText(replacement, from, to)
          }
          if (dispatch) dispatch(tr.setMeta(searchKey, { term: ps.term, matches: [], current: -1 }))
          return true
        },
    }
  },

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: searchKey,
        state: {
          init(): SearchState {
            return { term: '', matches: [], current: -1 }
          },
          apply(tr, prev): SearchState {
            const meta = tr.getMeta(searchKey)
            if (meta) return meta
            if (tr.docChanged && prev.term) {
              return { ...prev, matches: findMatches(tr.doc, prev.term) }
            }
            return prev
          },
        },
        props: {
          decorations(state) {
            const ps = searchKey.getState(state)!
            if (!ps.term || !ps.matches.length) return DecorationSet.empty
            const decos = ps.matches.map((m, i) =>
              Decoration.inline(m.from, m.to, {
                class: i === ps.current ? 'search-current' : 'search-match',
              })
            )
            return DecorationSet.create(state.doc, decos)
          },
        },
      }),
    ]
  },
})
