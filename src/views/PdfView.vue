<script setup lang="ts">
import { ref, shallowRef, computed, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import * as pdfjsLib from 'pdfjs-dist'
import { useAnnotations, type AnnotationInput } from '../composables/useAnnotations'
import type { Annotation } from '../composables/useAnnotations'

// @ts-ignore — Vite resolves this at build time
import pdfjsWorkerUrl from 'pdfjs-dist/build/pdf.worker.min.mjs?url'
pdfjsLib.GlobalWorkerOptions.workerSrc = pdfjsWorkerUrl

// ── Route ──────────────────────────────────────────────────────────────────────

const route  = useRoute()
const router = useRouter()

const attachmentId = computed(() => Number(route.query.id))
const itemId       = computed(() => Number(route.query.itemId))
const fileName     = computed(() => (route.query.name as string) || '')
const pdfTitle     = computed(() => (route.query.title as string) || fileName.value || 'PDF Viewer')

// ── Browser state ──────────────────────────────────────────────────────────────

interface AttachmentWithItem {
  id:           number
  itemId:       number
  fileName:     string
  addedAt:      number
  itemTitle?:   string
  itemAuthors?: string
  itemYear?:    string
  itemKey:      string
}

interface AttGroup {
  itemId:      number
  title:       string
  sub:         string
  attachments: AttachmentWithItem[]
}

const showBrowser    = ref(false)
const allAttachments = ref<AttachmentWithItem[]>([])

const groupedAttachments = computed<AttGroup[]>(() => {
  const map = new Map<number, AttGroup>()
  for (const a of allAttachments.value) {
    if (!map.has(a.itemId)) {
      map.set(a.itemId, {
        itemId:      a.itemId,
        title:       a.itemTitle ?? a.itemKey,
        sub:         [shortAuthors(a.itemAuthors), a.itemYear].filter(Boolean).join(' · '),
        attachments: [],
      })
    }
    map.get(a.itemId)!.attachments.push(a)
  }
  return [...map.values()]
})

function shortAuthors(authors?: string): string {
  if (!authors) return ''
  const parts = authors.split(/[,;&]|and\s/i).map(s => s.trim()).filter(Boolean)
  if (!parts.length) return ''
  const last = parts[0].split(/\s+/).pop() ?? parts[0]
  return parts.length > 2 ? `${last} et al.`
    : parts.length === 2 ? `${last} & ${parts[1].split(/\s+/).pop() ?? parts[1]}`
    : last
}

async function openAttachment(att: AttachmentWithItem) {
  const sameFile = att.id === attachmentId.value && !!pdfDoc.value
  showBrowser.value = false
  router.replace({ path: '/pdf', query: {
    id:     att.id,
    itemId: att.itemId,
    name:   att.fileName,
    title:  att.itemTitle ?? att.itemKey,
  }})
  if (sameFile) {
    // attachmentId didn't change → watch won't fire → re-render into the freshly mounted canvas
    await nextTick()
    await renderPage(currentPage.value)
    renderAnnotationOverlay()
  }
}

// ── PDF state ──────────────────────────────────────────────────────────────────

// shallowRef prevents Vue from Proxy-wrapping the pdfjs document instance.
// pdfjs v5 uses private class fields (#field) which break when accessed through a Proxy.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const pdfDoc      = shallowRef<any>(null)
const currentPage = ref(1)
const totalPages  = ref(0)
const scale       = ref(1.5)
const loading     = ref(false)
const pdfError    = ref('')

// ── DOM refs ───────────────────────────────────────────────────────────────────

const pageContainerEl = ref<HTMLElement | null>(null)
const canvasEl        = ref<HTMLCanvasElement | null>(null)
const textLayerEl     = ref<HTMLElement | null>(null)
const annLayerEl      = ref<HTMLElement | null>(null)

// ── Annotations ────────────────────────────────────────────────────────────────

const { annotations, loadAnnotations, createAnnotation, deleteAnnotation, updateAnnotationNote } = useAnnotations()

interface FloatingToolbar {
  x: number
  y: number
  text: string
  rects: Array<{ x1: number; y1: number; x2: number; y2: number }>
}
const floatingToolbar = ref<FloatingToolbar | null>(null)
const editingNoteId   = ref<number | null>(null)
const editingNoteText = ref('')

const ANN_COLORS = [
  { hex: '#FACC15', label: 'Yellow' },
  { hex: '#60A5FA', label: 'Blue'   },
  { hex: '#4ADE80', label: 'Green'  },
  { hex: '#F472B6', label: 'Pink'   },
]

const pageAnnotations = computed(() =>
  annotations.value.filter(a => a.page === currentPage.value)
)

const LAST_PDF_KEY = 'quire:lastPdf'

// ── PDF loading ────────────────────────────────────────────────────────────────

async function loadPdf() {
  if (!attachmentId.value) {
    loading.value = false
    return
  }
  loading.value = true
  pdfError.value = ''
  try {
    const bytes = await invoke<number[]>('read_attachment_bytes', { id: attachmentId.value })
    const uint8 = new Uint8Array(bytes)
    pdfDoc.value = await pdfjsLib.getDocument({ data: uint8 }).promise
    totalPages.value = pdfDoc.value.numPages
    localStorage.setItem(LAST_PDF_KEY, JSON.stringify({
      id:     attachmentId.value,
      itemId: itemId.value,
      name:   fileName.value,
      title:  pdfTitle.value,
      page:   1,
    }))
    loading.value = false
    await nextTick()
    const startPage = route.query.page ? Math.max(1, Number(route.query.page)) : 1
    await renderPage(startPage)
    await loadAnnotations(attachmentId.value)
    renderAnnotationOverlay()
  } catch (e) {
    pdfError.value = String(e)
    loading.value = false
  }
}

// ── Page rendering ─────────────────────────────────────────────────────────────

async function renderPage(pageNum: number) {
  if (!pdfDoc.value || !canvasEl.value || !textLayerEl.value || !pageContainerEl.value) return
  currentPage.value = pageNum
  floatingToolbar.value = null

  // Persist current page so "resume where left off" works
  try {
    const saved = localStorage.getItem(LAST_PDF_KEY)
    if (saved) {
      const data = JSON.parse(saved)
      localStorage.setItem(LAST_PDF_KEY, JSON.stringify({ ...data, page: pageNum }))
    }
  } catch { /* ignore */ }

  const page     = await pdfDoc.value.getPage(pageNum)
  const viewport = page.getViewport({ scale: scale.value })

  const canvas = canvasEl.value
  const ctx    = canvas.getContext('2d')!
  canvas.width  = viewport.width
  canvas.height = viewport.height
  pageContainerEl.value.style.width  = viewport.width  + 'px'
  pageContainerEl.value.style.height = viewport.height + 'px'
  await page.render({ canvasContext: ctx, viewport }).promise

  const textLayer = textLayerEl.value
  textLayer.innerHTML = ''
  textLayer.style.setProperty('--total-scale-factor', String(scale.value))
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const tl = new (pdfjsLib as any).TextLayer({
      textContentSource: page.streamTextContent(),
      container: textLayer,
      viewport,
    })
    await tl.render()
  } catch (e) {
    console.warn('[pdf] text layer:', e)
  }
  textLayer.style.width  = viewport.width  + 'px'
  textLayer.style.height = viewport.height + 'px'

  await nextTick()
  renderAnnotationOverlay()
}

// ── Annotation overlay ─────────────────────────────────────────────────────────

function renderAnnotationOverlay() {
  if (!annLayerEl.value) return
  annLayerEl.value.innerHTML = ''
  for (const ann of pageAnnotations.value) {
    let pos: { rects: Array<{ x1: number; y1: number; x2: number; y2: number }> }
    try { pos = JSON.parse(ann.positionJson) } catch { continue }
    for (const r of pos.rects ?? []) {
      const div = document.createElement('div')
      const isUnderline = ann.annType === 'underline'
      div.style.cssText = `
        position:absolute;
        left:${r.x1 * 100}%;
        top:${r.y1 * 100}%;
        width:${(r.x2 - r.x1) * 100}%;
        height:${(r.y2 - r.y1) * 100}%;
        background:${isUnderline ? 'transparent' : ann.color + '55'};
        border-bottom:${isUnderline ? '2px solid ' + ann.color : 'none'};
        pointer-events:none;
      `
      annLayerEl.value.appendChild(div)
    }
  }
}

watch(pageAnnotations, renderAnnotationOverlay)

// ── Text selection → floating toolbar ─────────────────────────────────────────

function onMouseUp(event: MouseEvent) {
  if ((event.target as HTMLElement).closest('.ann-toolbar')) return

  const sel = window.getSelection()
  if (!sel || sel.isCollapsed || sel.rangeCount === 0 || !textLayerEl.value) {
    floatingToolbar.value = null
    return
  }
  const range = sel.getRangeAt(0)
  if (!textLayerEl.value.contains(range.commonAncestorContainer)) {
    floatingToolbar.value = null
    return
  }

  const container = pageContainerEl.value!
  const cRect     = container.getBoundingClientRect()
  const rects: FloatingToolbar['rects'] = []
  let minTop = Infinity

  for (const r of Array.from(range.getClientRects())) {
    if (r.width < 1) continue
    rects.push({
      x1: (r.left   - cRect.left) / cRect.width,
      y1: (r.top    - cRect.top)  / cRect.height,
      x2: (r.right  - cRect.left) / cRect.width,
      y2: (r.bottom - cRect.top)  / cRect.height,
    })
    if (r.top < minTop) minTop = r.top
  }
  if (!rects.length) return

  floatingToolbar.value = {
    x: event.clientX,
    y: minTop - 52,
    text: sel.toString().trim(),
    rects,
  }
}

async function createHighlight(color: string, annType = 'highlight') {
  if (!floatingToolbar.value) return
  const input: AnnotationInput = {
    itemId:       itemId.value,
    attachmentId: attachmentId.value,
    page:         currentPage.value,
    annType,
    color,
    selectedText: floatingToolbar.value.text,
    positionJson: JSON.stringify({ rects: floatingToolbar.value.rects }),
  }
  await createAnnotation(input)
  window.getSelection()?.removeAllRanges()
  floatingToolbar.value = null
  renderAnnotationOverlay()
}

// ── Navigation ─────────────────────────────────────────────────────────────────

function prevPage() { if (currentPage.value > 1) renderPage(currentPage.value - 1) }
function nextPage() { if (currentPage.value < totalPages.value) renderPage(currentPage.value + 1) }
function zoomIn()  { scale.value = Math.min(scale.value + 0.25, 3); renderPage(currentPage.value) }
function zoomOut() { scale.value = Math.max(scale.value - 0.25, 0.5); renderPage(currentPage.value) }

// ── Inline note editing ────────────────────────────────────────────────────────

function startEditNote(ann: Annotation) {
  editingNoteId.value   = ann.id
  editingNoteText.value = ann.noteText ?? ''
}
async function saveNote(id: number) {
  await updateAnnotationNote(id, editingNoteText.value)
  editingNoteId.value = null
}

// ── Delete annotation ──────────────────────────────────────────────────────────

async function removeAnnotation(id: number) {
  await deleteAnnotation(id)
  renderAnnotationOverlay()
}

// ── Jump to annotation page ────────────────────────────────────────────────────

function jumpToPage(page: number) {
  if (page !== currentPage.value) renderPage(page)
}

// ── Standalone note ────────────────────────────────────────────────────────────

async function addStandaloneNote() {
  const input: AnnotationInput = {
    itemId:       itemId.value,
    attachmentId: attachmentId.value,
    page:         currentPage.value,
    annType:      'note',
    color:        '#FACC15',
    positionJson: JSON.stringify({ rects: [] }),
  }
  const ann = await createAnnotation(input)
  await nextTick()
  startEditNote(ann)
}

// ── Mount / unmount ────────────────────────────────────────────────────────────

onMounted(async () => {
  document.addEventListener('mouseup', onMouseUp)

  // Always pre-load the attachment list so the browser is ready instantly
  try {
    allAttachments.value = await invoke<AttachmentWithItem[]>('get_all_attachments_with_item')
  } catch { /* no attachments yet */ }

  if (!attachmentId.value) {
    try {
      const saved = localStorage.getItem(LAST_PDF_KEY)
      if (saved) {
        const { id, itemId: iid, name, title: t, page: p } = JSON.parse(saved)
        router.replace({ path: '/pdf', query: { id, itemId: iid, name, title: t, page: p ?? 1 } })
        return  // watch(attachmentId) fires loadPdf once query is set
      }
    } catch { /* corrupt entry — fall through to browser */ }
    showBrowser.value = true
    return
  }
  loadPdf()
})

onBeforeUnmount(() => {
  document.removeEventListener('mouseup', onMouseUp)
  if (pdfDoc.value) { pdfDoc.value.destroy(); pdfDoc.value = null }
})

watch(attachmentId, () => {
  if (attachmentId.value) loadPdf()
})
</script>

<template>
  <div class="pdf-view">

    <!-- ── Header ────────────────────────────────────────────────────────────── -->
    <div class="pdf-header">

      <!-- "← All PDFs" shown only when a PDF is loaded / loading -->
      <button v-if="!showBrowser" class="back-btn" @click="showBrowser = true">
        <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor"
             stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
          <path d="M8 2L3 6.5l5 4.5"/>
        </svg>
        All PDFs
      </button>

      <span class="pdf-title">{{ showBrowser ? 'All PDFs' : pdfTitle }}</span>

      <!-- PDF controls — only in viewer mode -->
      <div v-if="!showBrowser" class="pdf-controls">
        <button class="ctrl-btn" @click="zoomOut" title="Zoom out">−</button>
        <span class="zoom-label">{{ Math.round(scale * 100) }}%</span>
        <button class="ctrl-btn" @click="zoomIn"  title="Zoom in">+</button>
        <div class="page-nav">
          <button class="ctrl-btn" @click="prevPage" :disabled="currentPage <= 1">‹</button>
          <span class="page-counter">
            <input
              class="page-input"
              type="number"
              :value="currentPage"
              :min="1"
              :max="totalPages"
              @change="renderPage(Math.max(1, Math.min(totalPages, Number(($event.target as HTMLInputElement).value))))"
            />
            <span class="page-sep">/ {{ totalPages }}</span>
          </span>
          <button class="ctrl-btn" @click="nextPage" :disabled="currentPage >= totalPages">›</button>
        </div>
      </div>
    </div>

    <!-- ── Browser ────────────────────────────────────────────────────────────── -->
    <div v-if="showBrowser" class="pdf-browser">

      <div v-if="allAttachments.length === 0" class="browser-empty">
        <svg width="36" height="36" viewBox="0 0 36 36" fill="none" stroke="currentColor"
             stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" opacity="0.3">
          <rect x="7" y="3" width="22" height="30" rx="2"/>
          <path d="M12 11h12M12 17h12M12 23h8"/>
        </svg>
        <p>No PDFs yet</p>
        <p class="browser-empty-hint">Attach a PDF to a library item to see it here</p>
      </div>

      <div v-else class="browser-list">
        <div
          v-for="group in groupedAttachments"
          :key="group.itemId"
          class="browser-group"
        >
          <!-- Group header: clickable only when there's exactly one PDF (saves a click) -->
          <div
            class="browser-group-header"
            :class="{ 'header-clickable': group.attachments.length === 1 }"
            @click="group.attachments.length === 1 && openAttachment(group.attachments[0])"
          >
            <div class="browser-group-title">{{ group.title }}</div>
            <div class="browser-group-sub">{{ group.sub }}</div>
            <svg
              v-if="group.attachments.length === 1"
              class="header-arrow"
              width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor"
              stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M4 2l4 4-4 4"/>
            </svg>
            <span v-else class="browser-group-count">{{ group.attachments.length }} files</span>
          </div>

          <!-- Sub-rows when item has multiple PDFs -->
          <template v-if="group.attachments.length > 1">
            <button
              v-for="att in group.attachments"
              :key="att.id"
              class="browser-att-row"
              @click="openAttachment(att)"
            >
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor"
                   stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" class="att-icon">
                <rect x="2" y="1" width="8" height="10" rx="1"/>
                <path d="M4 4h4M4 6h4M4 8h2"/>
              </svg>
              <span class="att-name">{{ att.fileName }}</span>
              <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor"
                   stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="att-arrow">
                <path d="M3 2l4 3-4 3"/>
              </svg>
            </button>
          </template>
        </div>
      </div>
    </div>

    <!-- ── Viewer ─────────────────────────────────────────────────────────────── -->
    <div v-else class="pdf-body">

      <!-- PDF canvas area -->
      <div class="pdf-scroll">
        <div v-if="loading" class="pdf-loading">Loading…</div>
        <div v-else-if="pdfError" class="pdf-error">{{ pdfError }}</div>
        <div v-else-if="!pdfDoc" class="pdf-empty-state">
          <svg width="40" height="40" viewBox="0 0 40 40" fill="none" stroke="currentColor"
               stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" opacity="0.3">
            <rect x="8" y="4" width="24" height="32" rx="2"/>
            <path d="M14 13h12M14 19h12M14 25h8"/>
          </svg>
          <p>No PDF open</p>
        </div>
        <div v-else class="page-wrap">
          <div class="page-container" ref="pageContainerEl" @dragstart.prevent>
            <canvas ref="canvasEl"></canvas>
            <div class="text-layer" ref="textLayerEl"></div>
            <div class="ann-layer"  ref="annLayerEl"></div>
          </div>
        </div>
      </div>

      <!-- Floating annotation toolbar -->
      <div
        v-if="floatingToolbar"
        class="ann-toolbar"
        :style="{ left: floatingToolbar.x + 'px', top: floatingToolbar.y + 'px' }"
        @mousedown.prevent
      >
        <button
          v-for="c in ANN_COLORS"
          :key="c.hex"
          class="ann-color-btn"
          :style="{ background: c.hex }"
          :title="'Highlight ' + c.label"
          @click="createHighlight(c.hex, 'highlight')"
        ></button>
        <button class="ann-underline-btn" title="Underline" @click="createHighlight('#0A5FBF', 'underline')">U̲</button>
        <button class="ann-close-btn" @click="floatingToolbar = null">×</button>
      </div>

      <!-- Right panel: annotations list -->
      <div class="ann-panel">
        <div class="ann-panel-header">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor"
               stroke-width="1.5" stroke-linecap="round">
            <path d="M1 2h10M1 5h10M1 8h6"/>
          </svg>
          Annotations
          <span class="ann-count" v-if="annotations.length > 0">{{ annotations.length }}</span>
          <button
            v-if="attachmentId"
            class="add-note-btn"
            @click="addStandaloneNote"
            title="Add note for this page"
          >+ Note</button>
        </div>

        <div v-if="annotations.length === 0" class="ann-empty">
          Select text to highlight, or click <strong>+ Note</strong> to add a page note.
        </div>

        <div class="ann-list">
          <div
            v-for="ann in annotations"
            :key="ann.id"
            class="ann-item"
            :class="{ 'current-page': ann.page === currentPage }"
            @click="jumpToPage(ann.page)"
          >
            <div class="ann-item-header">
              <span class="ann-color-dot" :style="{ background: ann.color }"></span>
              <span class="ann-page-tag">p.{{ ann.page }}</span>
              <span class="ann-type-tag">{{ ann.annType }}</span>
              <button class="ann-del-btn" @click.stop="removeAnnotation(ann.id)" title="Delete">×</button>
            </div>
            <p v-if="ann.selectedText" class="ann-quote">"{{ ann.selectedText }}"</p>
            <template v-if="editingNoteId === ann.id">
              <textarea
                v-model="editingNoteText"
                class="ann-note-input"
                placeholder="Add a note…"
                @blur="saveNote(ann.id)"
                @keydown.enter.exact.prevent="saveNote(ann.id)"
                autofocus
              ></textarea>
              <p class="ann-note-hint">Enter to save · Shift+Enter for new line</p>
            </template>
            <p
              v-else
              class="ann-note"
              :class="{ empty: !ann.noteText }"
              @click.stop="startEditNote(ann)"
              title="Click to edit note"
            >{{ ann.noteText || 'Add note…' }}</p>
          </div>
        </div>
      </div>
    </div>

  </div>
</template>

<style scoped>
.pdf-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: var(--bg);
}

/* ── Header ──────────────────────────────────────────────────────────────────── */

.pdf-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-chrome);
  flex-shrink: 0;
}

.back-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  background: none;
  border: none;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-secondary);
  padding: 4px 6px;
  border-radius: var(--radius-sm);
  transition: background var(--t), color var(--t);
  flex-shrink: 0;
}
.back-btn:hover { background: var(--bg-document); color: var(--text); }

.pdf-title {
  flex: 1;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pdf-controls {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.ctrl-btn {
  background: var(--surface-solid);
  border: 1px solid var(--border-medium);
  border-radius: var(--radius-sm);
  min-width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 14px;
  color: var(--text-secondary);
  padding: 0 4px;
  transition: background var(--t);
}
.ctrl-btn:hover:not(:disabled) { background: var(--bg-chrome); }
.ctrl-btn:disabled { opacity: 0.35; cursor: default; }

.zoom-label {
  font-size: 11px;
  color: var(--text-tertiary);
  min-width: 34px;
  text-align: center;
  font-variant-numeric: tabular-nums;
}

.page-nav { display: flex; align-items: center; gap: 4px; }

.page-counter { display: flex; align-items: center; gap: 3px; }

.page-input {
  width: 36px;
  height: 24px;
  border: 1px solid var(--border-medium);
  border-radius: 4px;
  background: var(--surface-solid);
  text-align: center;
  font-size: 11.5px;
  color: var(--text);
  font-variant-numeric: tabular-nums;
  padding: 0;
}
.page-input::-webkit-inner-spin-button { display: none; }

.page-sep { font-size: 11px; color: var(--text-tertiary); font-variant-numeric: tabular-nums; }

/* ── Browser ─────────────────────────────────────────────────────────────────── */

.pdf-browser {
  flex: 1;
  overflow-y: auto;
  background: var(--bg);
}

.browser-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 80px 40px;
  color: var(--text-tertiary);
  text-align: center;
}
.browser-empty p { margin: 0; font-size: 13px; }
.browser-empty-hint { font-size: 11.5px; opacity: 0.7; }

.browser-list { padding: 8px 0; }

.browser-group { border-bottom: 1px solid var(--border); }
.browser-group:last-child { border-bottom: none; }

.browser-group-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 11px 18px 10px;
  background: var(--bg);
}

.browser-group-header.header-clickable {
  cursor: pointer;
  transition: background var(--t);
}
.browser-group-header.header-clickable:hover {
  background: var(--bg-chrome);
}

.browser-group-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.browser-group-sub {
  font-size: 11px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.browser-group-count {
  font-size: 10.5px;
  color: var(--text-tertiary);
  background: var(--bg-chrome-active);
  border-radius: 9px;
  padding: 1px 7px;
  flex-shrink: 0;
}

.header-arrow {
  color: var(--text-tertiary);
  flex-shrink: 0;
  opacity: 0.5;
}

.browser-att-row {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 18px 7px 34px;
  background: none;
  border: none;
  text-align: left;
  cursor: pointer;
  border-top: 1px solid var(--border);
  transition: background var(--t);
  font-family: var(--font-ui);
}
.browser-att-row:hover { background: var(--bg-chrome); }

.att-icon { color: var(--text-tertiary); flex-shrink: 0; }

.att-name {
  flex: 1;
  font-size: 12px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.att-arrow { color: var(--text-tertiary); flex-shrink: 0; opacity: 0; transition: opacity var(--t); }
.browser-att-row:hover .att-arrow { opacity: 1; }

/* ── Viewer body ─────────────────────────────────────────────────────────────── */

.pdf-body {
  display: flex;
  flex: 1;
  overflow: hidden;
  position: relative;
}

.pdf-scroll {
  flex: 1;
  overflow: auto;
  background: var(--bg-document);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 24px;
  min-width: 0;
}

.pdf-loading, .pdf-error {
  font-size: 13px;
  color: var(--text-tertiary);
  margin-top: 48px;
}
.pdf-error { color: #E8650A; }

.pdf-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  margin-top: 80px;
  color: var(--text-tertiary);
}
.pdf-empty-state p { margin: 0; font-size: 13px; }

.page-wrap { box-shadow: var(--shadow-lg); }

.page-container {
  position: relative;
  display: inline-block;
}
.page-container canvas { display: block; z-index: 1; }

.text-layer {
  position: absolute;
  top: 0; left: 0;
  overflow: clip;
  pointer-events: auto;
  -webkit-user-select: text;
  user-select: text;
  cursor: text;
  z-index: 2;
  line-height: 1;
  text-size-adjust: none;
  forced-color-adjust: none;
  --total-scale-factor: 1;
  --text-scale-factor: calc(var(--total-scale-factor) * var(--min-font-size, 1));
}

.text-layer :deep(span),
.text-layer :deep(br) {
  color: transparent;
  position: absolute;
  white-space: pre;
  cursor: text;
  transform-origin: 0% 0%;
  -webkit-user-select: text;
  user-select: text;
}

.text-layer :deep(span) {
  --font-height: 0;
  font-size: calc(var(--text-scale-factor) * var(--font-height));
  --scale-x: 1;
  --rotate: 0deg;
  transform: rotate(var(--rotate)) scaleX(var(--scale-x));
}

.text-layer :deep(span)::selection,
.text-layer :deep(::selection) {
  background: rgba(10, 95, 191, 0.3);
  color: transparent;
}

.ann-layer {
  position: absolute;
  top: 0; left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 3;
}

/* ── Floating annotation toolbar ─────────────────────────────────────────────── */

.ann-toolbar {
  position: fixed;
  z-index: 200;
  display: flex;
  align-items: center;
  gap: 5px;
  background: #1C1C1E;
  border-radius: 20px;
  padding: 6px 10px;
  box-shadow: 0 4px 20px rgba(0,0,0,0.35), 0 1px 4px rgba(0,0,0,0.2);
  transform: translateX(-50%);
  animation: toolbar-pop 0.1s ease-out;
}

@keyframes toolbar-pop {
  from { opacity: 0; transform: translateX(-50%) scale(0.88); }
  to   { opacity: 1; transform: translateX(-50%) scale(1); }
}

.ann-toolbar::after {
  content: '';
  width: 1px;
  height: 16px;
  background: rgba(255,255,255,0.15);
  margin: 0 2px;
}

.ann-color-btn {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: transform 0.1s, box-shadow 0.1s;
  box-shadow: 0 1px 3px rgba(0,0,0,0.3);
}
.ann-color-btn:hover { transform: scale(1.2); box-shadow: 0 2px 6px rgba(0,0,0,0.4); }

.ann-underline-btn, .ann-close-btn {
  background: none;
  border: none;
  cursor: pointer;
  color: rgba(255,255,255,0.7);
  font-size: 13px;
  padding: 0 2px;
  line-height: 1;
  transition: color 0.1s;
}
.ann-underline-btn:hover { color: #fff; }
.ann-close-btn:hover { color: #FF6B6B; }

/* ── Annotation panel ─────────────────────────────────────────────────────────── */

.ann-panel {
  width: 260px;
  flex-shrink: 0;
  border-left: 1px solid var(--border);
  background: var(--bg-chrome);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.ann-panel-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px 8px 14px;
  font-size: 10.5px;
  font-weight: 600;
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.add-note-btn {
  margin-left: auto;
  background: none;
  border: 1px solid var(--border-medium);
  border-radius: 4px;
  padding: 2px 7px;
  font-size: 10px;
  font-weight: 500;
  color: var(--text-secondary);
  cursor: pointer;
  white-space: nowrap;
  transition: background var(--t), color var(--t), border-color var(--t);
}
.add-note-btn:hover { background: var(--bg-document); border-color: var(--accent); color: var(--accent); }

.ann-count {
  background: var(--accent);
  color: #fff;
  font-size: 9.5px;
  font-weight: 700;
  border-radius: 8px;
  padding: 1px 5px;
  margin-left: 2px;
}

.ann-empty {
  font-size: 11.5px;
  color: var(--text-tertiary);
  padding: 16px 14px;
  font-style: italic;
  line-height: 1.5;
}

.ann-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.ann-item {
  background: var(--surface-solid);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 8px 9px;
  cursor: pointer;
  transition: box-shadow var(--t), border-color var(--t);
}
.ann-item:hover { box-shadow: var(--shadow-xs); }
.ann-item.current-page { border-color: var(--accent); }

.ann-item-header { display: flex; align-items: center; gap: 5px; margin-bottom: 5px; }

.ann-color-dot { width: 9px; height: 9px; border-radius: 50%; flex-shrink: 0; }
.ann-page-tag  { font-size: 10px; color: var(--text-tertiary); font-variant-numeric: tabular-nums; }
.ann-type-tag  { font-size: 9.5px; color: var(--text-tertiary); text-transform: uppercase; letter-spacing: 0.03em; }

.ann-del-btn {
  margin-left: auto;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-tertiary);
  font-size: 14px;
  line-height: 1;
  padding: 0;
  transition: color var(--t);
}
.ann-del-btn:hover { color: #E8650A; }

.ann-quote {
  font-size: 11px;
  color: var(--text-secondary);
  font-style: italic;
  margin: 0 0 4px;
  line-height: 1.4;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
}

.ann-note {
  font-size: 11px;
  color: var(--text-secondary);
  margin: 0;
  line-height: 1.4;
  cursor: text;
}
.ann-note.empty { color: var(--text-tertiary); font-style: italic; }

.ann-note-input {
  width: 100%;
  min-height: 56px;
  font-size: 11px;
  font-family: var(--font-ui);
  color: var(--text);
  background: var(--bg);
  border: 1px solid var(--accent);
  border-radius: 4px;
  padding: 4px 6px;
  resize: vertical;
  box-sizing: border-box;
}

.ann-note-hint {
  font-size: 9.5px;
  color: var(--text-tertiary);
  margin: 3px 0 0;
  font-style: italic;
}
</style>
