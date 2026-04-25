<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import { CitationNode } from '../extensions/CitationNode'
import { CitationSuggest } from '../extensions/CitationSuggest'
import { emitter } from '../events'
import { useDocument, type BibEntry } from '../composables/useDocument'
import { useFileOps } from '../composables/useFileOps'

const router = useRouter()
const { docTitle, docAuthors, citations, isDirty } = useDocument()
const { openDocument, saveDocument } = useFileOps()

// ── Editor ────────────────────────────────────────────────────────────────────

const INITIAL_CONTENT = `
<h2>Abstract</h2>
<p>The inadequate labelling of allergens in packaged foods poses significant public health risks, particularly for the estimated 220–520 million people globally affected by food allergies. This cross-sectional study examines allergen labelling compliance among packaged food manufacturers in India under the Food Safety and Standards (Labelling and Display) Regulations 2020, with comparative reference to EU Regulation 1169/2011 and the US FALCPA. We find systematic gaps in precautionary labelling practice and identify regulatory interpretation as a primary source of non-compliance.</p>
<h2>1. Introduction</h2>
<p>Food allergy affects an estimated 1 in 8 consumers <span data-cite-key="fda2021" data-index="2"></span> in developed markets, with rising prevalence documented across South Asian populations. Despite regulatory frameworks mandating allergen disclosure, recent systematic reviews indicate that approximately 34% of packaged food products <span data-cite-key="popova2022" data-index="1"></span> show discrepancies between declared allergen content and actual ingredient composition.</p>
<p>In the Indian context, the FSSAI has established a threshold of 10 mg/kg for individual allergen declarations under §4.2.1 of the 2020 labelling regulations <span data-cite-key="hadley2019" data-index="3"></span> yet enforcement mechanisms remain inconsistently applied across food category segments.</p>
<p>The foundational challenge is not merely one of regulatory compliance, but of communication design: precautionary allergen labels such as "may contain" are routinely misinterpreted by consumers, <span data-cite-key="hadley2019" data-index="3"></span> undermining their protective function even where they are accurately applied.</p>
<h2>2. Literature Review</h2>
<p>The landscape of allergen labelling research is characterised by a tension between regulatory prescription and real-world consumer behaviour. Foundational work by Hadley &amp; King (2019) <span data-cite-key="hadley2019" data-index="3"></span> established baseline comprehension rates for precautionary labelling across demographically stratified cohorts, finding that education level and prior allergy diagnosis are the primary moderators of label interpretation accuracy.</p>
<p>Subsequent systematic review of manufacturing-side compliance by Popova et al. (2022) <span data-cite-key="popova2022" data-index="1"></span> extended this analysis to the supply chain, demonstrating that discrepancies originate as frequently in ingredient sourcing and cross-contact risk management as in the labelling design itself.</p>
<h2>3. Methods</h2>
<p>Cross-sectional analysis of n=340 SKUs sampled from organised retail in three Indian metro markets (Chennai, Pune, Hyderabad). Audit conducted against FSSAI 2020 labelling regulations with comparative coding against EU and US frameworks. Inter-rater reliability: κ = 0.87. [Draft continues…]</p>
`

const editor = useEditor({
  extensions: [
    StarterKit.configure({
      heading: { levels: [2, 3] },
      horizontalRule: false,
      codeBlock: false,
      code: false,
    }),
    CitationNode,
    CitationSuggest,
  ],
  content: INITIAL_CONTENT,
  onUpdate() {
    isDirty.value = true
  },
})

// ── Tooltip state ─────────────────────────────────────────────────────────────

const tooltipVisible = ref(false)
const tooltipCitation = ref<BibEntry | null>(null)
const tooltipStyle = ref({ left: '0px', top: '0px' })
const tooltipHovered = ref(false)
let hideTimer: ReturnType<typeof setTimeout> | null = null

function findCitation(key: string) {
  return citations.value.find(c => c.key === key) ?? null
}

function startHide() {
  hideTimer = setTimeout(() => {
    if (!tooltipHovered.value) {
      tooltipVisible.value = false
      tooltipCitation.value = null
    }
  }, 200)
}

function cancelHide() {
  if (hideTimer) { clearTimeout(hideTimer); hideTimer = null }
}

// ── Citation panel state ──────────────────────────────────────────────────────

const panelOpen = ref(false)
const activeCitation = ref<BibEntry | null>(null)

// ── Event wiring ──────────────────────────────────────────────────────────────

// ── Keyboard shortcuts ────────────────────────────────────────────────────────

async function handleKeydown(e: KeyboardEvent) {
  const ctrl = e.ctrlKey || e.metaKey
  if (!ctrl) return
  if (e.key === 's') {
    e.preventDefault()
    const content = editor.value?.getHTML() ?? ''
    await saveDocument(content)
  }
  if (e.key === 'o') {
    e.preventDefault()
    const body = await openDocument()
    if (body && editor.value) {
      editor.value.commands.setContent(body)
      isDirty.value = false
    }
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)

  // Load content when a file is opened from the hamburger menu or file ops
  emitter.on('doc:opened', ({ content }) => {
    if (editor.value && content) {
      editor.value.commands.setContent(content)
      isDirty.value = false
    }
  })

  emitter.on('cite:hover', ({ key, rect }) => {
    if (panelOpen.value) return
    cancelHide()
    const cite = findCitation(key)
    if (!cite) return
    tooltipStyle.value = {
      left: `${rect.left + rect.width / 2}px`,
      top: `${rect.bottom + 10}px`,
    }
    tooltipCitation.value = cite
    tooltipVisible.value = true
  })

  emitter.on('cite:leave', () => {
    startHide()
  })

  emitter.on('cite:click', ({ key }) => {
    cancelHide()
    tooltipVisible.value = false
    const cite = findCitation(key)
    if (!cite) return
    activeCitation.value = cite
    panelOpen.value = true
  })
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
  emitter.off('doc:opened')
  emitter.off('cite:hover')
  emitter.off('cite:leave')
  emitter.off('cite:click')
  editor.value?.destroy()
  if (hideTimer) clearTimeout(hideTimer)
})

function closePanel() {
  panelOpen.value = false
  setTimeout(() => { activeCitation.value = null }, 260)
}

function openTooltipCitation() {
  if (!tooltipCitation.value) return
  activeCitation.value = tooltipCitation.value
  tooltipVisible.value = false
  panelOpen.value = true
}

function goToPdf() {
  router.push('/pdf')
}
</script>

<template>
  <div class="write-layout">
    <!-- Document scroll area -->
    <div class="document-area">
      <div class="paper">
        <!-- Paper header — mirrors placeholder exactly -->
        <div class="paper-eyebrow">Working Draft · April 2026</div>
        <h1 class="paper-title">{{ docTitle }}</h1>
        <p class="paper-subtitle">A Cross-Sectional Study of FSSAI Compliance and Consumer Risk Communication</p>
        <div class="paper-byline">
          <template v-for="(author, i) in docAuthors" :key="i">
            <span>{{ author }}</span>
            <span v-if="i < docAuthors.length - 1" class="by-sep">·</span>
          </template>
          <span class="by-sep">·</span>
          <span>Food Policy Studies, IIT Madras</span>
        </div>
        <div class="paper-rule"></div>

        <!-- Tiptap editor — content starts from Abstract -->
        <EditorContent :editor="editor" class="editor-body" />
      </div>
    </div>

    <!-- Citation panel (slide in) -->
    <Transition name="panel">
      <div class="citation-panel" v-if="panelOpen">
        <div class="cp-header">
          <span class="cp-label">Source</span>
          <button class="cp-close" @click="closePanel" title="Close">
            <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round">
              <line x1="1" y1="1" x2="12" y2="12"/>
              <line x1="12" y1="1" x2="1" y2="12"/>
            </svg>
          </button>
        </div>
        <div class="cp-body" v-if="activeCitation">
          <p class="cp-title">{{ activeCitation.title }}</p>
          <div class="cp-meta">
            <span class="cp-authors">{{ activeCitation.authors }}</span>
            <span class="cp-journal">{{ activeCitation.journal }} · {{ activeCitation.year }}</span>
            <span class="cp-doi">{{ activeCitation.doi }}</span>
          </div>
          <div class="cp-block">
            <div class="cp-block-label">Abstract</div>
            <p class="cp-abstract">{{ activeCitation.abstractText }}</p>
          </div>
          <button class="cp-detail-btn" @click="goToPdf">
            View in Detail
            <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
              <path d="M2 6.5h9M7.5 2.5l4 4-4 4"/>
            </svg>
          </button>
        </div>
      </div>
    </Transition>
  </div>

  <!-- Tooltip -->
  <Teleport to="body">
    <div
      class="cite-tooltip"
      v-show="tooltipVisible && tooltipCitation"
      :style="tooltipStyle"
      @mouseenter="tooltipHovered = true; cancelHide()"
      @mouseleave="tooltipHovered = false; startHide()"
    >
      <template v-if="tooltipCitation">
        <div class="tt-title">{{ tooltipCitation.title }}</div>
        <div class="tt-authors">{{ tooltipCitation.authors }} · {{ tooltipCitation.year }}</div>
        <div class="tt-excerpt">{{ (tooltipCitation.abstractText ?? '').slice(0, 130) }}…</div>
        <button class="tt-cta" @click="openTooltipCitation">View in detail →</button>
      </template>
    </div>
  </Teleport>
</template>

<style scoped>
.write-layout {
  display: flex;
  height: 100%;
  overflow: hidden;
}

.document-area {
  flex: 1;
  overflow-y: auto;
  background: var(--bg-document);
  display: flex;
  justify-content: center;
  padding: 40px 28px 80px;
  min-width: 0;
}

.paper {
  width: 100%;
  max-width: 660px;
  background: var(--surface-solid);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow);
  padding: 52px 60px 64px;
  border: 1px solid rgba(0,0,0,0.05);
  height: fit-content;
}

.paper-eyebrow {
  font-size: 10.5px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--text-tertiary);
  margin-bottom: 14px;
}

.paper-title {
  font-family: var(--font-doc);
  font-size: 23px;
  font-weight: 700;
  line-height: 1.3;
  letter-spacing: -0.02em;
  color: var(--text);
  margin-bottom: 6px;
}

.paper-subtitle {
  font-family: var(--font-doc);
  font-size: 14.5px;
  font-style: italic;
  color: var(--text-secondary);
  margin-bottom: 18px;
  line-height: 1.5;
}

.paper-byline {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 24px;
}

.by-sep {
  color: var(--text-tertiary);
}

.paper-rule {
  height: 1px;
  background: var(--border);
  margin-bottom: 28px;
}

.editor-body {
  /* EditorContent fills remaining space; ProseMirror styled in editor.css */
}

/* Citation panel — identical to placeholder */
.citation-panel {
  width: 336px;
  flex-shrink: 0;
  border-left: 3px solid var(--accent);
  background: var(--surface);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: -4px 0 20px rgba(0,0,0,0.06);
}

.cp-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px 10px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.cp-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  color: var(--text-tertiary);
}

.cp-close {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-tertiary);
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  transition: background var(--t), color var(--t);
}
.cp-close:hover {
  background: var(--bg-chrome-active);
  color: var(--text);
}

.cp-body {
  padding: 18px 16px 20px;
  overflow-y: auto;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.cp-title {
  font-family: var(--font-doc);
  font-size: 14.5px;
  font-weight: 700;
  line-height: 1.45;
  color: var(--text);
}

.cp-meta {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.cp-authors {
  font-size: 12px;
  font-weight: 500;
  color: var(--text);
}

.cp-journal {
  font-size: 11px;
  color: var(--text-secondary);
}

.cp-doi {
  font-family: var(--font-mono);
  font-size: 10px;
  color: var(--text-tertiary);
}

.cp-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.cp-block-label {
  font-size: 9.5px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  color: var(--text-tertiary);
}

.cp-abstract {
  font-family: var(--font-doc);
  font-size: 13px;
  line-height: 1.65;
  color: var(--text-secondary);
}

.cp-detail-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius);
  padding: 9px 14px;
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;
  transition: opacity var(--t);
  margin-top: auto;
  flex-shrink: 0;
}
.cp-detail-btn:hover { opacity: 0.86; }

.panel-enter-active,
.panel-leave-active {
  transition: transform var(--t), opacity var(--t);
}
.panel-enter-from,
.panel-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>

<!-- Tooltip global styles — not scoped, teleported to body -->
<style>
.cite-tooltip {
  position: fixed;
  transform: translateX(-50%);
  z-index: 9999;
  background: rgba(255,255,255,0.97);
  backdrop-filter: blur(28px);
  -webkit-backdrop-filter: blur(28px);
  border: 1px solid rgba(0,0,0,0.09);
  border-radius: 10px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.12), 0 2px 6px rgba(0,0,0,0.06);
  padding: 14px 15px;
  width: 275px;
  pointer-events: auto;
}

.tt-title {
  font-family: Georgia, "Times New Roman", serif;
  font-size: 13px;
  font-weight: 700;
  color: #1A1917;
  line-height: 1.4;
  margin-bottom: 4px;
}

.tt-authors {
  font-size: 11px;
  color: #6B6A68;
  margin-bottom: 8px;
}

.tt-excerpt {
  font-family: Georgia, "Times New Roman", serif;
  font-size: 12px;
  line-height: 1.55;
  color: #6B6A68;
  margin-bottom: 10px;
}

.tt-cta {
  background: none;
  border: none;
  padding: 0;
  font-size: 11.5px;
  font-weight: 600;
  color: #0A5FBF;
  cursor: pointer;
  font-family: -apple-system, BlinkMacSystemFont, sans-serif;
}

.tt-cta:hover { text-decoration: underline; }
</style>
