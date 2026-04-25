# Quire — Integrated Research Environment

A desktop writing environment for researchers who work with citations, annotations, and source-heavy prose. Built with Tauri 2, Vue 3, and TypeScript.

---

## The problem it solves

Writing a cited paragraph currently means switching between your editor, a PDF reader, a reference manager, and a notes app — often several times per sentence. Quire keeps citations, source annotations, and prose in one place, so you can write and verify in the same window.

## M0 feature set (current)

- **WYSIWYG `.qmd` editor** — ProseMirror via Tiptap, Georgia body type, section headings rendered inline
- **Inline citations** — `[1]` `[2]` superscripts backed by real `.bib` keys; hover for a quick card, click to open a full side panel
- **`@` citation autocomplete** — type `@` anywhere in the editor to get an Overleaf-style dropdown of your bibliography; filter by key, title, or author with ↑↓ to navigate and Enter/click to insert
- **Unresolved citation indicator** — any `citeKey` with no matching entry in the `.bib` renders as orange `[?]` with a tooltip naming the missing key
- **Global bibliography** — `~/.quire/references.bib` is the single source of truth across all projects; seeded with sample entries on first run
- **Hamburger menu** — `≡` in the title bar opens a quick-access panel with New Document, Open File, and a Recent Files list with relative timestamps
- **Open / Save / Save As** — native file dialogs via `tauri-plugin-dialog`; documents saved as `.qmd` with YAML frontmatter
- **Quarto export** — "Export as PDF" invokes `quarto render` via `tauri-plugin-shell`; requires Quarto CLI in PATH
- **Workbench view** — annotation blocks from multiple sources with colour-coded source tags (placeholder data, M1 wires real Zotero annotations)
- **PDF viewer** — split layout: simulated PDF page on the left, citing-drafts + annotation cards on the right (placeholder, M2 wires real PDF.js)

## Tech stack

| Layer | Library |
|---|---|
| Desktop shell | Tauri 2 |
| Frontend framework | Vue 3 + TypeScript (Vite 6) |
| Editor | Tiptap v2 (ProseMirror) |
| Routing | vue-router 4 |
| Events | mitt |
| File dialogs | tauri-plugin-dialog |
| Shell execution | tauri-plugin-shell |
| Home dir (Rust) | dirs 5 |
| Citation autocomplete | @tiptap/suggestion |

## Getting started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) ≥ 18
- [Tauri CLI v2](https://tauri.app/start/prerequisites/)
- [Quarto CLI](https://quarto.org/docs/get-started/) (optional — only needed for PDF export)

### Development

```bash
npm install
npm run tauri dev
```

The app opens at its own window (not a browser tab). Hot-reload is active for the Vue frontend; Rust changes require a recompile.

### Production build

```bash
npm run tauri build
```

Outputs a native installer in `src-tauri/target/release/bundle/`.

## Project layout

```
quire/
├── src/                      # Vue frontend
│   ├── assets/               # Global CSS (style.css, editor.css)
│   ├── components/           # TitleBar, Sidebar, StatusBar, HamburgerMenu,
│   │                         # CitationNodeView, CitationTooltip, CitationPanel
│   ├── composables/          # useDocument, useFileOps, useQuire
│   ├── events.ts             # mitt event bus (cite:*, doc:*, export:*)
│   ├── extensions/           # CitationNode Tiptap extension
│   ├── router/               # vue-router routes
│   └── views/                # WriteView, WorkbenchView, PdfView
└── src-tauri/                # Rust backend
    └── src/lib.rs            # Tauri commands: open/save, .bib parsing, recent files
```

## Data storage

On first launch Quire creates `~/.quire/` containing:

- `references.bib` — global bibliography, pre-seeded with four sample entries
- `recent.json` — list of recently opened files (max 15, managed automatically)

Every project reads from `~/.quire/references.bib` by default. A local `.bib` file in the same directory as a `.qmd` document will take precedence if present.

## Roadmap

See [`ROADMAP.md`](ROADMAP.md) for the full milestone breakdown (M0–M6).

## Development setup

- VS Code + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
