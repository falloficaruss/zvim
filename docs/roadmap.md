# ZVIM Roadmap

## Phase 0: Foundation

Outcome:

- workspace exists
- module boundaries are clear
- performance goals are written down

Done in this repo:

- Rust workspace scaffold
- product thesis
- architecture and roadmap docs

## Phase 1: Fast Editor Core

Build:

- rope-backed buffer
- cursor and selection engine
- insert and normal modes
- undo and redo transactions
- viewport abstraction
- command model designed for native modal editing

Success criteria:

- basic editing feels crisp
- large files do not cause obvious stalls
- command handling is testable without UI

## Phase 2: GPUI App Shell

Build:

- application bootstrap
- one clean IDE-style window with one editor surface
- input routing
- rendering of text, cursor, and selections
- command palette and file picker
- project tree and workspace navigation
- first-class settings surface with live reload where possible

Success criteria:

- open file and edit loop works smoothly
- UI can redraw incrementally
- editor state is not trapped in GPUI components
- users can change important preferences without the product feeling brittle
- the workspace feels clean rather than crowded

## Phase 3: Language Intelligence

Build:

- LSP manager
- diagnostics
- hover
- completion
- rename and go-to-definition

Success criteria:

- language servers can fail without freezing the app
- diagnostics update incrementally
- completion latency feels competitive

## Phase 4: Built-In Git

Build:

- gutter diff indicators
- inline diff visibility
- stage and revert by hunk
- stage and unstage by file
- blame on demand
- branch awareness in the workspace

Success criteria:

- common Git workflows feel native and low-friction
- Git operations do not stall editing or painting
- the built-in Git experience is useful without becoming noisy

## Phase 5: Power Features

Build:

- multi-cursor workflows
- tree-sitter syntax tree awareness
- structural editing
- fast search across large repos
- git diff and inline blame

Success criteria:

- advanced editing feels native, not layered on
- repo navigation is fast enough to trust every day

## Phase 6: ZVIM-Specific Advantages

Build:

- deeply integrated modal workflows
- programmable commands
- remote development and collaboration primitives
- advanced configuration depth for expert users

Success criteria:

- ZVIM has a clear identity
- users can explain why it is better, not just different
- expert users feel unusually empowered without the defaults becoming overwhelming

## Phase 7: Community Extension Platform

Build:

- stable extension API
- extension host runtime
- sandboxed permissions model
- publishing and marketplace flow
- compatibility and performance guidelines for community packages

Success criteria:

- third parties can build useful extensions without private internal access
- broken or slow extensions do not stall the UI
- the ecosystem expands ZVIM's reach without weakening the built-in experience

## Order Of Attack

The recommended order is:

1. build the editing engine
2. build the config system
3. attach GPUI to the engine
4. add tree-first workspace navigation
5. add built-in Git
6. add language services
7. add advanced workflows
8. deepen differentiation features
9. open the community extension platform

That order gives us the best chance of making something fast, durable, and hard to outgrow.
