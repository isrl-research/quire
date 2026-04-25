import mitt from 'mitt'

export type Events = {
  'cite:hover': { key: string; rect: DOMRect }
  'cite:leave': void
  'cite:click': { key: string }
  'doc:saved': { path: string }
  'doc:opened': { path: string; content: string }
  'bib:loaded': void
  'export:start': void
  'export:done': { outputPath: string }
  'export:error': { message: string }
}

export const emitter = mitt<Events>()
