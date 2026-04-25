# CLAUDE.md — Quire project context for future sessions

This file exists so you can pick up exactly where we left off without needing conversation history. Read it before touching anything. The codebase is the ground truth for current state; this file is the ground truth for *why*.

---

## What Quire is and why it exists

Quire is a desktop writing environment for academic researchers. The name is deliberate: *a quire* is the unit of folded leaves that makes a book — the thing before the thing is bound.

It was born from a specific, documented pain session: writing a paper on allergen labelling in packaged foods. During a single editorial session, the author had to:

- Open an annotated DOCX a supervisor had returned
- Cross-reference citation numbers across 7 PDFs
- Switch between Zotero, a browser, Overleaf, and a terminal
- Recover a figure from an old git commit because it had been accidentally deleted
- Look up an ORCID manually
- Realise mid-draft that two citations had been numbered wrong

**Eight tool switches to write three paragraphs.** That session is the user story. Every feature in Quire traces back to one of those switches.

The core insight is that **existing tools are built around claims management, not narration**. Zotero manages a reference library. Overleaf manages a LaTeX document. Obsidian manages a knowledge graph. None of them understand the act of writing a sentence that makes a claim, points at a source, and needs to be traceable to a page in that source. Quire is built around that act as the atomic unit.

---

## The philosophy

**Narrative-first, not claim-first.** Researchers write arguments in prose. The tools should make the prose surface the primary environment, with everything else (references, annotations, source PDFs, export) reachable without leaving. A citation in Quire is an inline object in the editor — not a footnote database entry you link to, not a LaTeX command you type blind.

**Local-first, file-system native.** Documents are `.qmd` files. The bibliography is a `.bib` file at `~/.quire/references.bib`. Everything lives on disk in standard formats that work without Quire installed. There is no Quire account, no cloud sync, no proprietary format.

**Single source of truth for references.** All projects share `~/.quire/references.bib`. You curate one library, not one per project. Per-project override is possible (a local `.bib` in the same directory takes precedence), but the global file is the default.

**The workbench is the missing layer.** Between "I have annotations from 4 papers" and "I am writing a paragraph that synthesises them" there is a cognitive step that every tool currently punts to the researcher's head or a scratch document. Quire's Workbench view makes that step explicit and spatial: annotation blocks from your sources, organised against your draft outline, with source provenance on every block.

---

## Where we are: M0 complete

M0 is done and working. The branch is `feat/m0`, PR exists to `main`. What's shipped:

### Real editor
- Tiptap (ProseMirror) WYSIWYG editor in `WriteView.vue`
- Document content is `.qmd` (Quarto Markdown) with YAML frontmatter parsed by the Rust backend
- Georgia serif body, system-ui chrome — mirrors the exact aesthetic of the placeholder mockups

### Citations as first-class editor objects
- `CitationNode` — a custom Tiptap inline atom node that stores `citeKey` + `displayIndex`
- Rendered as a blue `[N]` superscript by `CitationNodeView.vue` (Vue NodeView)
- Hover → frosted-glass tooltip with title, authors, abstract excerpt (no panel opens)
- Click → side panel slides in with full source detail
- `[?]` in orange if the `citeKey` has no matching entry in the loaded `.bib`

### `@` citation autocomplete
- Type `@` anywhere in the editor → Overleaf-style dropdown appears
- Searches the loaded `.bib` by key, title, or author as you type
- ↑↓ navigate, Enter or click to insert; sequential `[N]` auto-assigned
- Implemented via `@tiptap/suggestion` + `CitationSuggest.ts` extension

### Global bibliography
- `~/.quire/references.bib` seeded on first launch with 4 sample entries (Popova 2022, FDA 2021, Hadley & King 2019, FSSAI 2023 — all from the allergen paper session)
- Rust backend has a hand-rolled `.bib` parser (no crates — handles braced/quoted/nested values, multi-author fields)
- Commands: `get_global_bib`, `save_global_bib`, `get_global_bib_raw`, `find_bib_for_document`

### File operations
- Open / Save / Save As via `tauri-plugin-dialog`
- YAML frontmatter written on save (title, authors, date)
- Frontmatter parsed on open — populates `docTitle`, `docAuthors` in the document store
- Recent files tracked in `~/.quire/recent.json` (max 15, Unix-seconds timestamps)

### Hamburger menu
- `≡` button in the TitleBar opens a slide-in panel
- New Document, Open File, and a Recent Files list with relative timestamps (`2h ago` etc.)
- Outside-click and Escape to dismiss

### Navigation views (placeholder data, real UI)
- **Workbench** (`/workbench`) — annotation blocks with coloured source tags, draft outline
- **PDF** (`/pdf`) — simulated split: PDF page left, citing-drafts + annotations right

### Quarto export
- "Export as PDF" button invokes `quarto render` via `tauri-plugin-shell`
- Requires Quarto CLI in PATH; wired up but not tested end-to-end

---

## Architecture decisions and why they were made

### Module-level reactive singletons instead of Pinia
`useDocument()` returns module-level `ref()`s that are shared across all component instances. Avoids the Pinia boilerplate for a project this size. The composable is a thin wrapper around the refs — every caller gets the same underlying reactive objects. Do not refactor this to Pinia without a clear reason; the current approach is intentional.

### mitt event bus for editor ↔ component communication
Tiptap NodeViews are isolated from the parent component's template scope. There is no clean Vue way to bind events from inside a ProseMirror NodeView to the parent. The pattern is: NodeView fires `emitter.emit('cite:hover', ...)`, `WriteView.vue` listens in `onMounted`. This is correct and intentional — do not try to replace it with props/emits.

Key events:
- `cite:hover { key, rect }` — fired on citation mouseenter
- `cite:leave` — fired on citation mouseleave
- `cite:click { key }` — fired on citation click
- `doc:opened { path, content }` — fired when a file loads (editor resets content)
- `doc:saved { path }` — fired on successful save
- `export:start / export:done / export:error` — for the status bar

### Hash history router
`createWebHashHistory()` is used because Tauri serves from a file URL, not a real origin. `createWebHistory()` breaks in production Tauri builds. Do not change this.

### Rust `.bib` parser (no crates)
The parser in `src-tauri/src/lib.rs` handles `.bib` correctly without `biblatex` or `nom` crates. It splits on `@`, reads depth-tracked braces/quotes for values, handles nested braces in abstracts. It was written by hand because adding a `.bib` parsing crate to the Tauri binary caused build complexity that wasn't worth it at M0. The parser is not perfect (it does not handle all edge cases) but it handles the common journal article entry format used by Zotero exports.

### `.qmd` as document format
Quarto Markdown is the document format. It's plain Markdown with YAML frontmatter, and Quarto can render it to PDF, DOCX, HTML. The editor saves HTML from Tiptap and the Rust backend writes it as `.qmd`. This is a simplification — real `.qmd` stores Markdown, not HTML — and will need revisiting in M1 when the export pipeline needs to produce correct Quarto output.

### Mac aesthetic with plain CSS
No Tailwind, no component library, no CSS framework. All styles are in `src/assets/style.css` (global tokens) and `src/assets/editor.css` (Tiptap-specific). This is intentional — the app targets macOS aesthetic precisely, and framework utilities would fight the fine-grained control the design requires.

Key design tokens in `style.css`:
- `--bg: #F5F4F1` — warm off-white, document background
- `--bg-chrome: #EDECEA` — slightly cooler, used for titlebar/sidebar
- `--bg-document: #E8E6E0` — the document canvas behind the paper card
- `--surface-solid: #FFFFFF` — the paper card itself
- `--accent: #0A5FBF` — citation blue, primary interactive colour
- `--accent-orange: #E8650A` — unresolved citation `[?]`
- `--font-doc: Georgia, "Times New Roman", serif` — body text in editor
- `--font-ui: -apple-system, BlinkMacSystemFont, "Segoe UI Variable", sans-serif`

---

## Key files map

```
src/
├── assets/
│   ├── style.css              ← Global CSS variables. Change tokens here to retheme.
│   └── editor.css             ← ProseMirror/Tiptap styles. .cite-sup-node lives here.
├── events.ts                  ← mitt event bus. All event types are typed here.
├── composables/
│   ├── useDocument.ts         ← THE reactive store. Module-level singletons. Read this first.
│   ├── useFileOps.ts          ← open/save/saveAs/tryLoadBib. Bridges Tauri commands ↔ store.
│   └── useQuire.ts            ← initQuire (loads global .bib), recent files, relativeTime.
├── extensions/
│   ├── CitationNode.ts        ← Tiptap node definition. Attributes: citeKey, displayIndex.
│   └── CitationSuggest.ts     ← @ autocomplete extension. Uses @tiptap/suggestion.
├── components/
│   ├── CitationNodeView.vue   ← Renders each [N] in the editor. Fires cite:* events.
│   ├── CitationSuggestList.vue ← The @ dropdown popup. Exposes onKeyDown to extension.
│   ├── HamburgerMenu.vue      ← ≡ menu: new doc, open file, recent files.
│   ├── TitleBar.vue           ← Traffic lights + title + export button + ≡ trigger.
│   ├── Sidebar.vue            ← 52px icon strip, 3 route links.
│   └── StatusBar.vue          ← Word count, save status, dirty indicator.
├── views/
│   ├── WriteView.vue          ← The editor. Most complex file. Tiptap + panels + events.
│   ├── WorkbenchView.vue      ← Placeholder. Real implementation is M1.
│   └── PdfView.vue            ← Placeholder. Real implementation is M2.
└── App.vue                    ← Shell: titlebar, sidebar, statusbar, hamburger, router-view.

src-tauri/src/lib.rs           ← All Rust commands. .bib parser, .qmd read/write, recent files.
~/.quire/references.bib        ← The global bibliography. Single source of truth.
~/.quire/recent.json           ← Recent files list. [ { path, title, last_opened } ]
```

---

## What's next (M1 priorities, in order)

1. **Fix the .qmd save format** — currently saves Tiptap's HTML output, which isn't valid `.qmd`. The save pipeline needs to convert Tiptap's HTML to Markdown (using Tiptap's `generateText` or a ProseMirror serialiser), wrap in YAML frontmatter, and write that. The editor should load `.qmd` by parsing the Markdown back to Tiptap HTML on open.

2. **Zotero annotation import** — Zotero stores PDF annotations in `~/Zotero/zotero.sqlite`. Read highlighted text + page number + source paper per highlight, surface them as workbench blocks. No API required — just SQLite reads.

3. **Real Workbench** — replace the placeholder `WorkbenchView.vue` with actual annotation blocks sourced from (2). Drag-to-draft: dragging a block into the editor inserts a blockquote with the cite-key attached.

4. **Quarto export test** — verify the `run_quarto` Rust command actually works end-to-end with a real `.qmd` file. The command exists; it has not been tested against a real document.

5. **Auto-save** — save every 60s if dirty, not just on Ctrl+S.

---

## What to never do

- **Don't add Pinia.** The singleton composable pattern is intentional.
- **Don't change hash history to web history.** Production Tauri builds break.
- **Don't add a CSS framework.** The design requires precise control; utilities fight it.
- **Don't store document content in the Rust backend state.** The frontend owns document state; Rust just reads and writes files on demand.
- **Don't add Zotero API integration.** The design decision is file-watching the local Zotero SQLite and Better BibTeX exports — no auth, no sync, no API key.
- **Don't touch WorkbenchView or PdfView until M1/M2.** They are correct placeholders; premature implementation before the data layer exists would produce throwaway code.

---

## The user

The user is a researcher/developer at IIT Madras working on food policy and allergen labelling. They have prior experience with Tauri and are comfortable with Rust, Vue, and TypeScript at an intermediate level. They care deeply about the UX details — the "Mac aesthetic" is not a preference, it is a constraint. When in doubt, match macOS system UI: native blur, muted borders, traffic-light window controls, Georgia serif for document body, system-ui for chrome.

The project is personal and research-motivated. Decisions should be made to serve the actual allergen paper session workflow, not to build a general-purpose tool.
