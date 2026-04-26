<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import Heading from '@tiptap/extension-heading'
import { Mathematics } from '@tiptap/extension-mathematics'
import 'katex/dist/katex.min.css'
import { CitationNode } from '../extensions/CitationNode'
import { CitationSuggest } from '../extensions/CitationSuggest'
import { SearchAndReplace, searchKey } from '../extensions/SearchAndReplace'
import { SectionRef } from '../extensions/SectionRef'
import { SectionRefSuggest } from '../extensions/SectionRefSuggest'
import { emitter } from '../events'
import { useDocument, type BibEntry, type DocAuthor } from '../composables/useDocument'
import { useFileOps } from '../composables/useFileOps'
import { openUrl } from '@tauri-apps/plugin-opener'

// Heading extended with a stable UUID `id` attribute for section cross-references
const ExtendedHeading = Heading.configure({ levels: [1, 2, 3] }).extend({
  addAttributes() {
    return {
      ...this.parent?.(),
      id: {
        default: null,
        parseHTML: (el: Element) => el.getAttribute('data-hid') ?? null,
        renderHTML: (attrs: any) => attrs.id ? { 'data-hid': attrs.id } : {},
      },
    }
  },
})

const router = useRouter()
const { docTitle, docSubtitle, docStatus, docDate, docAuthors, citations, isDirty } = useDocument()
const { openDocument, saveDocument } = useFileOps()

// ── Editor ────────────────────────────────────────────────────────────────────

const INITIAL_CONTENT = `
<p><strong>Abstract.</strong> The inadequate labelling of allergens in packaged foods poses significant public health risks, particularly for the estimated 220–520 million people globally affected by food allergies. This cross-sectional study examines allergen labelling compliance among packaged food manufacturers in India under the Food Safety and Standards (Labelling and Display) Regulations 2020, with comparative reference to EU Regulation 1169/2011 and the US FALCPA. We find systematic gaps in precautionary labelling practice and identify regulatory interpretation as a primary source of non-compliance.</p>
<h1>Introduction</h1>
<p>Food allergy affects an estimated 1 in 8 consumers <span data-cite-key="fda2021" data-index="2"></span> in developed markets, with rising prevalence documented across South Asian populations. Despite regulatory frameworks mandating allergen disclosure, recent systematic reviews indicate that approximately 34% of packaged food products <span data-cite-key="popova2022" data-index="1"></span> show discrepancies between declared allergen content and actual ingredient composition.</p>
<p>In the Indian context, the FSSAI has established a threshold of 10 mg/kg for individual allergen declarations under §4.2.1 of the 2020 labelling regulations <span data-cite-key="hadley2019" data-index="3"></span> yet enforcement mechanisms remain inconsistently applied across food category segments.</p>
<p>The foundational challenge is not merely one of regulatory compliance, but of communication design: precautionary allergen labels such as "may contain" are routinely misinterpreted by consumers, <span data-cite-key="hadley2019" data-index="3"></span> undermining their protective function even where they are accurately applied.</p>
<h1>Literature Review</h1>
<p>The landscape of allergen labelling research is characterised by a tension between regulatory prescription and real-world consumer behaviour. Foundational work by Hadley &amp; King (2019) <span data-cite-key="hadley2019" data-index="3"></span> established baseline comprehension rates for precautionary labelling across demographically stratified cohorts, finding that education level and prior allergy diagnosis are the primary moderators of label interpretation accuracy.</p>
<p>Subsequent systematic review of manufacturing-side compliance by Popova et al. (2022) <span data-cite-key="popova2022" data-index="1"></span> extended this analysis to the supply chain, demonstrating that discrepancies originate as frequently in ingredient sourcing and cross-contact risk management as in the labelling design itself.</p>
<h1>Methods</h1>
<h2>Study design</h2>
<p>Cross-sectional analysis of <em>n</em> = 340 SKUs sampled from organised retail in three Indian metro markets (Chennai, Pune, Hyderabad). Audit conducted against FSSAI 2020 labelling regulations with comparative coding against EU and US frameworks. Inter-rater reliability: κ = 0.87.</p>
<h2>Statistical model</h2>
<p>Compliance rate across product categories was modelled as a binomial proportion. The 95% confidence interval for overall non-compliance was computed as $\\hat{p} \\pm 1.96\\sqrt{\\hat{p}(1-\\hat{p})/n}$. Display-mode example:</p>
$$\\kappa = \\frac{p_o - p_e}{1 - p_e}$$
<p>[Draft continues…]</p>
`

const editor = useEditor({
  extensions: [
    StarterKit.configure({
      heading: false,
      horizontalRule: false,
      codeBlock: false,
      code: false,
    }),
    ExtendedHeading,
    Mathematics.configure({
      blockOptions: { onClick: (node: any, pos: number) => openMathEdit(node, pos, 'blockMath') },
      inlineOptions: { onClick: (node: any, pos: number) => openMathEdit(node, pos, 'inlineMath') },
    }),
    CitationNode,
    CitationSuggest,
    SectionRef,
    SectionRefSuggest,
    SearchAndReplace,
  ],
  editorProps: {
    attributes: { spellcheck: 'true' },
  },
  content: INITIAL_CONTENT,
  onUpdate() {
    isDirty.value = true
  },
})

// ── Find & Replace ────────────────────────────────────────────────────────────

const findOpen = ref(false)
const showReplace = ref(false)
const findTerm = ref('')
const replaceTerm = ref('')
const srCount = ref(0)
const srCurrent = ref(-1)

function syncSrState() {
  const ps = editor.value ? searchKey.getState(editor.value.state) : null
  srCount.value = ps?.matches.length ?? 0
  srCurrent.value = ps?.current ?? -1
}

function openFind(withReplace = false) {
  findOpen.value = true
  showReplace.value = withReplace
  setTimeout(() => (document.querySelector('.find-input') as HTMLElement | null)?.focus(), 30)
}

function closeFind() {
  findOpen.value = false
  findTerm.value = ''
  replaceTerm.value = ''
  editor.value?.commands.setSearch('')
  srCount.value = 0
  srCurrent.value = -1
  editor.value?.commands.focus()
}

function onFindInput() {
  editor.value?.commands.setSearch(findTerm.value)
  syncSrState()
  if (srCount.value > 0) {
    editor.value?.commands.findNext()
    syncSrState()
  }
}

function doFindNext() {
  editor.value?.commands.findNext()
  syncSrState()
}

function doFindPrev() {
  editor.value?.commands.findPrev()
  syncSrState()
}

function doReplaceOne() {
  editor.value?.commands.replaceOne(replaceTerm.value)
  syncSrState()
}

function doReplaceAll() {
  editor.value?.commands.replaceAll(replaceTerm.value)
  syncSrState()
}

// ── Math edit overlay ─────────────────────────────────────────────────────────

interface MathEditState {
  visible: boolean
  latex: string
  pos: number
  type: 'blockMath' | 'inlineMath'
  style: { left: string; top: string }
}

const mathEdit = ref<MathEditState>({
  visible: false,
  latex: '',
  pos: 0,
  type: 'blockMath',
  style: { left: '0px', top: '0px' },
})

function openMathEdit(node: any, pos: number, type: 'blockMath' | 'inlineMath') {
  const dom = editor.value?.view.nodeDOM(pos) as HTMLElement | null
  const rect = dom?.getBoundingClientRect()
  mathEdit.value = {
    visible: true,
    latex: node.attrs.latex ?? '',
    pos,
    type,
    style: {
      left: `${rect ? Math.min(rect.left, window.innerWidth - 360) : 200}px`,
      top: `${rect ? rect.bottom + 8 : 200}px`,
    },
  }
  // focus the textarea after Vue renders
  setTimeout(() => {
    (document.querySelector('.math-edit-input') as HTMLElement | null)?.focus()
  }, 30)
}

function applyMathEdit() {
  if (!editor.value || !mathEdit.value.visible) return
  const { pos, type, latex } = mathEdit.value
  editor.value.chain().focus().setNodeSelection(pos).updateAttributes(type, { latex }).run()
  mathEdit.value.visible = false
}

function closeMathEdit() {
  mathEdit.value.visible = false
  editor.value?.commands.focus()
}

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
  if (e.key === 'Escape') {
    if (findOpen.value) { e.preventDefault(); closeFind(); return }
  }
  if (e.key === 'F11') { e.preventDefault(); toggleFocusMode(); return }
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
  if (e.key === 'f') {
    e.preventDefault()
    openFind(false)
  }
  if (e.key === 'h') {
    e.preventDefault()
    openFind(true)
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
  // Imperative scroll listener — more reliable than @scroll on the template ref
  documentAreaRef.value?.addEventListener('scroll', onDocScroll, { passive: true })

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
  documentAreaRef.value?.removeEventListener('scroll', onDocScroll)
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
  metaOpen.value = false
  panelOpen.value = true
}

function goToPdf() {
  router.push('/pdf')
}

// ── Metadata panel ────────────────────────────────────────────────────────────

const metaOpen = ref(false)

function openMeta() {
  panelOpen.value = false
  activeCitation.value = null
  metaOpen.value = true
}

function addAuthor() {
  docAuthors.value.push({ name: '', orcid: '', title: '', affiliation: '' })
}

function removeAuthor(i: number) {
  docAuthors.value.splice(i, 1)
}

// ── Focus mode ────────────────────────────────────────────────────────────────

const focusMode = ref(false)
const documentAreaRef = ref<HTMLElement | null>(null)

function toggleFocusMode() {
  focusMode.value = !focusMode.value
  if (focusMode.value) {
    updateDocAbove()
  } else {
    clearDocAbove()
  }
}

function getProseMirror(): HTMLElement | null {
  return documentAreaRef.value?.querySelector('.ProseMirror') as HTMLElement | null
}

function updateDocAbove() {
  const root = documentAreaRef.value
  const pm = getProseMirror()
  if (!root || !pm) return
  const rootRect = root.getBoundingClientRect()
  // Fade blocks whose bottom crosses into the top 30% of the scroll container.
  const threshold = rootRect.top + rootRect.height * 0.3
  Array.from(pm.children).forEach(child => {
    const el = child as HTMLElement
    const below = el.getBoundingClientRect().bottom >= threshold
    el.style.opacity = below ? '' : '0.2'
    el.style.transition = 'opacity 0.4s ease'
  })
}

function clearDocAbove() {
  const pm = getProseMirror()
  if (!pm) return
  Array.from(pm.children).forEach(child => {
    const el = child as HTMLElement
    el.style.opacity = ''
    el.style.transition = ''
  })
}

function onDocScroll() {
  if (focusMode.value) updateDocAbove()
}

// ── Byline helpers ────────────────────────────────────────────────────────────

const uniqueAffiliations = computed(() => {
  const seen = new Set<string>()
  const result: string[] = []
  for (const author of docAuthors.value) {
    const aff = author.affiliation.trim()
    if (aff && !seen.has(aff)) {
      seen.add(aff)
      result.push(aff)
    }
  }
  return result
})

function affiliationNumbers(author: DocAuthor): number[] {
  const aff = author.affiliation.trim()
  if (!aff) return []
  const idx = uniqueAffiliations.value.indexOf(aff)
  return idx >= 0 ? [idx + 1] : []
}

function openOrcid(orcid: string) {
  openUrl(`https://orcid.org/${orcid}`).catch(() => {})
}
</script>

<template>
  <div class="write-layout">
    <!-- Document scroll area -->
    <div
      class="document-area"
      :class="{ 'focus-mode': focusMode }"
      ref="documentAreaRef"
    >
      <div class="paper">
        <!-- Paper header -->
        <div class="paper-eyebrow">
          <span>{{ docStatus }}{{ docDate ? ' · ' + docDate : '' }}</span>
          <button
            class="meta-trigger"
            :class="{ 'focus-active': focusMode }"
            @click="toggleFocusMode"
            :title="focusMode ? 'Exit focus mode (F11)' : 'Focus mode (F11)'"
          >
            <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="7" cy="7" r="2.5"/>
              <circle cx="7" cy="7" r="5.5" :stroke-opacity="focusMode ? 1 : 0.45"/>
            </svg>
          </button>
          <button class="meta-trigger" @click="openMeta" title="Edit document metadata">
            <svg width="11" height="11" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M8.5 1.5a1.414 1.414 0 0 1 2 2L3 11H1v-2L8.5 1.5z"/>
            </svg>
          </button>
        </div>
        <h1 class="paper-title">{{ docTitle }}</h1>
        <p class="paper-subtitle" v-if="docSubtitle">{{ docSubtitle }}</p>
        <div class="paper-byline">
          <!-- Author names row -->
          <div class="byline-authors">
            <span v-for="(author, i) in docAuthors" :key="i" class="byline-author">
              <span class="author-name">{{ author.name }}</span>
              <sup v-if="uniqueAffiliations.length > 1 && affiliationNumbers(author).length" class="affil-sup">
                {{ affiliationNumbers(author).join(',') }}
              </sup>
              <a
                v-if="author.orcid"
                class="orcid-link"
                href="#"
                @click.prevent="openOrcid(author.orcid)"
                :title="`ORCID: ${author.orcid}`"
              >
                <svg class="orcid-logo" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                  <circle cx="12" cy="12" r="12" fill="#A6CE39"/>
                  <rect x="7" y="7.5" width="2.2" height="9.5" rx="1" fill="white"/>
                  <circle cx="8.1" cy="5.2" r="1.5" fill="white"/>
                  <path d="M11.5 7.5h4.1c2.8 0 4.4 2 4.4 4.75S18.4 17 15.6 17H11.5V7.5z" fill="white"/>
                </svg>
              </a>
              <span v-if="i < docAuthors.length - 1" class="by-sep"> · </span>
            </span>
          </div>
          <!-- Affiliations row -->
          <div v-if="uniqueAffiliations.length" class="byline-affiliations">
            <span v-for="(aff, i) in uniqueAffiliations" :key="i" class="byline-affil">
              <sup v-if="uniqueAffiliations.length > 1" class="affil-sup">{{ i + 1 }}</sup>{{ aff }}<span v-if="i < uniqueAffiliations.length - 1" class="by-sep">  ·  </span>
            </span>
          </div>
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

    <!-- Metadata panel (slide in) -->
    <Transition name="panel">
      <div class="citation-panel meta-panel" v-if="metaOpen">
        <div class="cp-header">
          <span class="cp-label">Document</span>
          <button class="cp-close" @click="metaOpen = false" title="Close">
            <svg width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round">
              <line x1="1" y1="1" x2="12" y2="12"/>
              <line x1="12" y1="1" x2="1" y2="12"/>
            </svg>
          </button>
        </div>
        <div class="cp-body mp-body">

          <!-- Title -->
          <div class="mp-group">
            <div class="mp-group-label">Title</div>
            <input class="mp-input" v-model="docTitle" type="text" placeholder="Paper title" />
          </div>

          <!-- Subtitle -->
          <div class="mp-group">
            <div class="mp-group-label">Subtitle</div>
            <input class="mp-input" v-model="docSubtitle" type="text" placeholder="Subtitle or description" />
          </div>

          <!-- Status + Date row -->
          <div class="mp-row">
            <div class="mp-group mp-half">
              <div class="mp-group-label">Status</div>
              <input class="mp-input" v-model="docStatus" type="text" placeholder="Working Draft" />
            </div>
            <div class="mp-group mp-half">
              <div class="mp-group-label">Date</div>
              <input class="mp-input" v-model="docDate" type="text" placeholder="April 2026" />
            </div>
          </div>

          <div class="mp-divider"></div>

          <!-- Authors -->
          <div class="mp-authors-header">
            <span class="mp-group-label" style="margin-bottom:0">Authors</span>
            <button class="mp-add-btn" @click="addAuthor">+ Add</button>
          </div>

          <div v-for="(author, i) in docAuthors" :key="i" class="mp-author-card">
            <div class="mp-author-row">
              <div class="mp-group" style="flex:1">
                <div class="mp-group-label">Name</div>
                <input class="mp-input" v-model="author.name" type="text" placeholder="Surname, Initials" />
              </div>
              <button v-if="docAuthors.length > 1" class="mp-remove-btn" @click="removeAuthor(i)" title="Remove author">
                <svg width="11" height="11" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round">
                  <line x1="1" y1="1" x2="12" y2="12"/><line x1="12" y1="1" x2="1" y2="12"/>
                </svg>
              </button>
            </div>
            <div class="mp-group">
              <div class="mp-group-label">ORCID</div>
              <input class="mp-input mp-mono" v-model="author.orcid" type="text" placeholder="0000-0000-0000-0000" />
            </div>
            <div class="mp-row">
              <div class="mp-group mp-half">
                <div class="mp-group-label">Position</div>
                <input class="mp-input" v-model="author.title" type="text" placeholder="PhD Candidate" />
              </div>
              <div class="mp-group mp-half">
                <div class="mp-group-label">Affiliation</div>
                <input class="mp-input" v-model="author.affiliation" type="text" placeholder="Institution" />
              </div>
            </div>
          </div>

        </div>
      </div>
    </Transition>
  </div>

  <!-- Find & Replace bar -->
  <Teleport to="body">
    <Transition name="find-bar">
      <div v-if="findOpen" class="find-bar">

        <!-- Find row -->
        <div class="find-row">
          <div class="find-input-wrap">
            <svg class="find-icon" width="13" height="13" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
              <circle cx="5.5" cy="5.5" r="4"/>
              <line x1="8.5" y1="8.5" x2="12" y2="12"/>
            </svg>
            <input
              class="find-input"
              v-model="findTerm"
              placeholder="Find…"
              @input="onFindInput"
              @keydown.enter.exact.prevent="doFindNext"
              @keydown.shift.enter.prevent="doFindPrev"
              @keydown.escape.prevent="closeFind"
            />
            <span class="find-count" v-if="findTerm">
              {{ srCount === 0 ? 'No results' : `${srCurrent >= 0 ? srCurrent + 1 : '?'} / ${srCount}` }}
            </span>
          </div>
          <button class="find-nav-btn" @click="doFindPrev" title="Previous (Shift+Enter)">
            <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
              <path d="M2 7l3.5-3.5L9 7"/>
            </svg>
          </button>
          <button class="find-nav-btn" @click="doFindNext" title="Next (Enter)">
            <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
              <path d="M2 4l3.5 3.5L9 4"/>
            </svg>
          </button>
          <button class="find-toggle-btn" :class="{ active: showReplace }" @click="showReplace = !showReplace" title="Toggle replace">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M1 4h7M5 2l3 2-3 2"/>
              <path d="M11 8H4M8 6l-3 2 3 2"/>
            </svg>
          </button>
          <button class="find-close-btn" @click="closeFind" title="Close (Esc)">
            <svg width="11" height="11" viewBox="0 0 13 13" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round">
              <line x1="1" y1="1" x2="12" y2="12"/><line x1="12" y1="1" x2="1" y2="12"/>
            </svg>
          </button>
        </div>

        <!-- Replace row -->
        <div class="find-row find-replace-row" v-if="showReplace">
          <div class="find-input-wrap">
            <input
              class="find-input"
              v-model="replaceTerm"
              placeholder="Replace with…"
              @keydown.enter.exact.prevent="doReplaceOne"
              @keydown.escape.prevent="closeFind"
            />
          </div>
          <button class="find-action-btn" @click="doReplaceOne" :disabled="srCount === 0">Replace</button>
          <button class="find-action-btn" @click="doReplaceAll" :disabled="srCount === 0">All</button>
        </div>

      </div>
    </Transition>
  </Teleport>

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

  <!-- Math edit overlay -->
  <Teleport to="body">
    <div
      v-if="mathEdit.visible"
      class="math-edit-overlay"
      :style="mathEdit.style"
    >
      <div class="math-edit-label">{{ mathEdit.type === 'blockMath' ? 'Display math' : 'Inline math' }}</div>
      <textarea
        v-model="mathEdit.latex"
        class="math-edit-input"
        :rows="mathEdit.type === 'blockMath' ? 3 : 1"
        @keydown.ctrl.enter.prevent="applyMathEdit"
        @keydown.meta.enter.prevent="applyMathEdit"
        @keydown.enter.exact.prevent="mathEdit.type === 'inlineMath' && applyMathEdit()"
        @keydown.escape.prevent="closeMathEdit"
      ></textarea>
      <div class="math-edit-hint">{{ mathEdit.type === 'inlineMath' ? 'Enter to apply' : 'Ctrl+Enter to apply' }} · Esc to cancel</div>
    </div>

    <!-- click outside to close math editor -->
    <div
      v-if="mathEdit.visible"
      style="position:fixed;inset:0;z-index:9997"
      @mousedown.self="closeMathEdit"
    ></div>
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
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10.5px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--text-secondary);
  margin-bottom: 14px;
}

.paper-eyebrow > span {
  flex: 1;
}

.meta-trigger {
  background: none;
  border: none;
  padding: 3px 5px;
  cursor: pointer;
  color: var(--text-tertiary);
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  opacity: 0.55;
  transition: opacity var(--t), background var(--t), color var(--t);
}

.meta-trigger:hover {
  opacity: 1;
  background: var(--bg-chrome-active);
  color: var(--text-secondary);
}

.meta-trigger.focus-active {
  opacity: 1;
  color: var(--accent-purple, #7C3AED);
}

.meta-trigger.focus-active:hover {
  color: var(--accent-purple, #7C3AED);
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
  flex-direction: column;
  gap: 5px;
  margin-bottom: 24px;
}

.byline-authors {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  font-size: 12.5px;
  color: var(--text);
  font-weight: 500;
}

.byline-author {
  display: inline-flex;
  align-items: baseline;
  gap: 1px;
  white-space: nowrap;
}

.author-name {
  letter-spacing: -0.01em;
}

.affil-sup {
  font-size: 8.5px;
  vertical-align: super;
  line-height: 0;
  color: var(--text-secondary);
  font-weight: 400;
  margin-left: 1px;
}

.orcid-link {
  display: inline-flex;
  align-items: center;
  margin-left: 3px;
  vertical-align: middle;
  text-decoration: none;
  opacity: 0.75;
  transition: opacity var(--t);
  position: relative;
  top: -0.5px;
}

.orcid-link:hover {
  opacity: 1;
}

.orcid-logo {
  width: 13px;
  height: 13px;
  display: block;
}

.byline-affiliations {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  font-size: 12px;
  color: var(--text-secondary);
  font-style: italic;
}

.byline-affil {
  display: inline;
}

.by-sep {
  color: var(--text-tertiary);
  font-style: normal;
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
  color: var(--text-secondary);
}

.cp-close {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
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
  font-size: 11px;
  color: var(--text-secondary);
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
  color: var(--text-secondary);
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

/* ── Metadata panel ──────────────────────────────────────────── */

.meta-panel {
  border-left-color: var(--border-medium);
}

.mp-body {
  gap: 12px;
}

.mp-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.mp-group-label {
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--text-secondary);
  margin-bottom: 2px;
}

.mp-input {
  background: var(--bg-chrome, #EDECEA);
  border: 1px solid var(--border, rgba(0,0,0,0.09));
  border-radius: 5px;
  padding: 6px 8px;
  font-family: var(--font-ui);
  font-size: 12.5px;
  color: var(--text);
  outline: none;
  width: 100%;
  box-sizing: border-box;
  transition: border-color var(--t);
}

.mp-input:focus {
  border-color: var(--accent);
}

.mp-input::placeholder {
  color: var(--text-tertiary);
}

.mp-mono {
  font-family: var(--font-mono, monospace);
  font-size: 11.5px;
  letter-spacing: 0.04em;
}

.mp-row {
  display: flex;
  gap: 8px;
}

.mp-half {
  flex: 1;
  min-width: 0;
}

.mp-divider {
  height: 1px;
  background: var(--border);
  margin: 4px 0;
}

.mp-authors-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.mp-add-btn {
  background: none;
  border: none;
  font-size: 11px;
  font-weight: 600;
  color: var(--accent);
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
  transition: background var(--t);
}

.mp-add-btn:hover {
  background: var(--accent-soft);
}

.mp-author-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px;
  background: var(--bg-chrome, #EDECEA);
  border-radius: 6px;
  border: 1px solid var(--border);
}

.mp-author-row {
  display: flex;
  align-items: flex-end;
  gap: 6px;
}

.mp-remove-btn {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  padding: 4px;
  border-radius: 4px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  margin-bottom: 1px;
  transition: background var(--t), color var(--t);
}

.mp-remove-btn:hover {
  background: rgba(196, 80, 0, 0.08);
  color: var(--accent-orange, #C45000);
}

/* override input background inside author card */
.mp-author-card .mp-input {
  background: var(--surface-solid, #fff);
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
  color: var(--text);
  line-height: 1.4;
  margin-bottom: 4px;
}

.tt-authors {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.tt-excerpt {
  font-family: Georgia, "Times New Roman", serif;
  font-size: 12px;
  line-height: 1.55;
  color: var(--text-secondary);
  margin-bottom: 10px;
}

.tt-cta {
  background: none;
  border: none;
  padding: 0;
  font-size: 11.5px;
  font-weight: 600;
  color: var(--accent);
  cursor: pointer;
  font-family: -apple-system, BlinkMacSystemFont, sans-serif;
}

.tt-cta:hover { text-decoration: underline; }

/* ── Find & Replace bar ──────────────────────────────────────── */

.find-bar {
  position: fixed;
  top: calc(var(--titlebar-h, 42px) + 10px);
  right: 28px;
  z-index: 600;
  background: rgba(255, 255, 255, 0.98);
  backdrop-filter: blur(28px);
  -webkit-backdrop-filter: blur(28px);
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 10px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12), 0 2px 8px rgba(0, 0, 0, 0.06);
  overflow: hidden;
  min-width: 360px;
}

.find-bar-enter-active,
.find-bar-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}
.find-bar-enter-from,
.find-bar-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

.find-row {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 8px 8px 10px;
}

.find-replace-row {
  border-top: 1px solid rgba(0, 0, 0, 0.07);
  padding-top: 7px;
  padding-bottom: 7px;
}

.find-input-wrap {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--bg-chrome, #EDECEA);
  border: 1px solid var(--border, rgba(0,0,0,0.09));
  border-radius: 6px;
  padding: 5px 8px;
  min-width: 0;
}

.find-icon {
  flex-shrink: 0;
  color: var(--text-secondary);
}

.find-input {
  flex: 1;
  border: none;
  background: none;
  outline: none;
  font-family: var(--font-ui);
  font-size: 13px;
  color: var(--text);
  min-width: 0;
}

.find-input::placeholder {
  color: var(--text-tertiary);
}

.find-count {
  flex-shrink: 0;
  font-family: var(--font-ui);
  font-size: 11px;
  font-weight: 500;
  color: var(--text-secondary);
  white-space: nowrap;
}

.find-nav-btn {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
  flex-shrink: 0;
  transition: background 0.15s, color 0.15s;
}
.find-nav-btn:hover {
  background: var(--bg-chrome, #EDECEA);
  color: var(--text);
}

.find-toggle-btn {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
  flex-shrink: 0;
  transition: background 0.15s, color 0.15s;
}
.find-toggle-btn:hover,
.find-toggle-btn.active {
  background: var(--accent-soft, rgba(10,95,191,0.08));
  color: var(--accent);
}

.find-close-btn {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
  flex-shrink: 0;
  transition: background 0.15s, color 0.15s;
}
.find-close-btn:hover {
  background: var(--bg-chrome, #EDECEA);
  color: var(--text);
}

.find-action-btn {
  background: none;
  border: 1px solid var(--border-medium, rgba(0,0,0,0.12));
  border-radius: 5px;
  padding: 4px 10px;
  font-family: var(--font-ui);
  font-size: 12px;
  font-weight: 500;
  color: var(--text);
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  transition: background 0.15s;
}
.find-action-btn:hover:not(:disabled) {
  background: var(--bg-chrome, #EDECEA);
}
.find-action-btn:disabled {
  opacity: 0.4;
  cursor: default;
}
</style>
