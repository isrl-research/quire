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

## Known gaps and honest state (assessed after M0)

This section captures an honest audit of what M0 actually delivers versus what it promises. Read before starting M1 work.

### Critical: the .qmd save format is broken
`useFileOps.ts` saves Tiptap's raw HTML output as the document body. A file saved by Quire is not valid Quarto Markdown — Quarto cannot render it, and it's unreadable in any other editor. **This is the first thing to fix in M1, before anything else.** The fix: add a ProseMirror HTML→Markdown serialiser on save, and a Markdown→Tiptap HTML parser on load. Tiptap's `@tiptap/pm` gives access to the ProseMirror model; a custom serialiser is ~100 lines. The citation node needs a special serialisation rule: `[@citeKey]` in Pandoc citation syntax.

### Citation numbering is static and will become friction
`displayIndex` is assigned at insertion time and never updated. If a user inserts a citation at the top of the document after already having `[1]`, `[2]`, `[3]` lower down, the numbers don't reorder. For M0 this is tolerable; in real use it becomes a constant annoyance. Fix: on every editor update, traverse the document, collect citation nodes in order, reassign indices by unique key in order of first appearance, update node attrs. This is a ProseMirror transaction-based operation.

### The workbench — the most novel feature — is entirely placeholder
The workbench is the genuinely differentiated idea: annotation blocks from multiple source PDFs, spatially organised against a draft outline, drag-to-insert into prose. No tool in the researcher's current stack does this. But `WorkbenchView.vue` is 100% hardcoded fake data. Until Zotero's SQLite is being read (M1), Quire's most important feature does not exist. The UI shell is correct; the data layer is missing entirely.

### Quarto export is wired but untested end-to-end
`run_quarto` exists in `lib.rs` and the export button is wired. It has never been run against a real document. It will also produce garbage output until the .qmd save format is fixed.

### Citation panel is read-only
You cannot add, edit, or delete `.bib` entries from within Quire. Adding a new reference requires editing `~/.quire/references.bib` externally and relaunching (or triggering a re-load). A minimal in-app "Add entry" form would close this gap; a full editor is probably overkill for M1.

### Missing basic academic editor features
Tables, figure captions, math/equations (critical for STEM fields), footnotes, and cross-references are all absent. These are Tiptap extensions and are addable incrementally; they are not architectural problems. Prioritise based on the allergen paper's actual content — it used tables and statistics.

### What genuinely works right now
- `@` autocomplete is immediately useful
- Citation hover/click panel is better than the Zotero equivalent for in-flow reference checking
- Global `.bib` design is sound
- `[?]` unresolved indicator is a good editorial guardrail
- The file open/save/recent files loop works correctly (modulo the HTML format issue)
- The aesthetic and shell are production-quality

---

## What's next (M1 priorities, in order)

1. **Fix the .qmd save format** — blocker for everything else. Write a ProseMirror→Markdown serialiser that handles paragraphs, headings, and citation nodes (serialise as `[@citeKey]` in Pandoc syntax). On load, parse Markdown back to Tiptap HTML. See "Known gaps" section above for detail.

2. **Dynamic citation index recomputation** — on every editor transaction, traverse document, collect citations in order of appearance, reassign `displayIndex` by unique key. Prevents numbering drift as the user writes.

3. **Quarto export end-to-end test** — once the save format is fixed, run `run_quarto` against a real `.qmd` and verify the PDF output is correct. Fix any issues in the Rust command.

4. **Zotero annotation import** — read `~/Zotero/zotero.sqlite` (no API, no auth). Extract: highlighted text, page number, colour, source paper (match to `.bib` key). Surface as workbench blocks.

5. **Real Workbench** — replace placeholder `WorkbenchView.vue` with annotation blocks from (4). Drag-to-draft inserts a blockquote with the cite-key attached.

6. **Auto-save** — save every 60s if dirty.

7. **Minimal in-app bib entry form** — a simple "Add reference" sheet (title, authors, year, key, journal, DOI) that appends to `~/.quire/references.bib`. Removes the need to edit the file externally.

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
