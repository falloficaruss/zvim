# ZVIM Architecture

## Design Goal

ZVIM should feel immediate. The architecture must preserve responsiveness even when syntax parsing, indexing, AI tooling, and language servers are all active.

## High-Level Shape

Split the system into four layers:

1. app shell
2. editor engine
3. background services
4. persistence and collaboration

### 1. App Shell

This is where GPUI belongs.

Responsibilities:

- windows and panels
- event routing
- layout and rendering
- focus management
- input handling
- presentation state

Non-responsibilities:

- buffer truth
- command semantics
- LSP process ownership
- long-running indexing work

The UI should observe domain state and dispatch commands, not own core editing behavior.

### 2. Editor Engine

This is the heart of the product.

Responsibilities:

- rope-backed buffers
- cursors and multi-cursors
- selections
- undo and redo
- modal state and commands
- viewport mapping
- transaction log for edits

This layer should be deterministic and heavily tested. If we ever need to switch UI frameworks, this layer should survive mostly unchanged.

### 3. Background Services

These services must run asynchronously and be isolated from frame-critical work.

Services:

- LSP client runtime
- tree-sitter parsing
- file indexing
- fuzzy search
- AI context building
- git status and diff computation

Every service should communicate through explicit messages or subscriptions. No service should be allowed to block input or painting.

### 4. Persistence and Collaboration

This layer handles:

- workspace config
- session restore
- keymaps
- command history
- remote sync
- collaborative editing

Keep it separate from editing primitives so recovery, sync, and replay remain feasible.

## Configuration System

ZVIM should support a broad settings surface for users who want deep control, but it should do so through a disciplined configuration system rather than ad hoc flags.

The configuration layer should cover:

- editor behavior
- keymaps and modal behavior
- rendering and typography
- panels and layout preferences
- language-specific settings
- AI and automation policies
- extension permissions and defaults

Design constraints:

- defaults should be strong enough that most users never need to tune everything
- settings should be layered cleanly across default, user, workspace, and project scopes
- settings changes should be observable and reversible
- invalid settings should fail clearly rather than silently corrupting behavior
- configuration reloads should avoid full app restarts whenever possible
- the settings system must not become a hidden dependency graph that hurts startup or interaction latency

## Extension Model

ZVIM should eventually support a community extension ecosystem in the spirit of VS Code, where outside developers can create and publish useful packages.

The key difference is sequencing. We should not build ZVIM as "just a shell for extensions" at the start. The right order is:

1. build an excellent core editor
2. define clean command and state boundaries
3. expose those boundaries through a stable extension API
4. open the ecosystem gradually

Architectural constraints:

- extensions cannot own core editor truth
- extensions should run in an isolated host or process boundary
- extension APIs should be explicit rather than exposing arbitrary internals
- rendering and input latency must not depend on extension behavior
- the core product should remain strong even with zero third-party extensions installed

## GPUI Strategy

Use GPUI as the UI runtime and renderer, not as the place where editor logic accumulates.

That means:

- views render projections of editor state
- commands flow into the editor engine
- expensive computations happen outside render loops
- the UI subscribes to granular state changes instead of refreshing whole scenes

## Performance Budget

Initial target budgets:

- cold startup to visible window: under 120 ms on a healthy dev machine
- keypress to painted frame: under 8 ms in typical files
- command dispatch overhead: under 4 ms
- opening a large file should degrade gracefully rather than freeze

## Suggested Near-Term Modules

`zvim-core`

- shared ids
- latency budgets
- command envelopes
- event types
- config schema primitives

`zvim-editor`

- buffer model
- selections
- history
- motions
- viewport

`zvim-lsp`

- process management
- request routing
- diagnostics model
- completion model

`zvim-app`

- GPUI app bootstrap
- window composition
- panel system
- key input mapping
- settings UI and live config application

## Risks To Avoid Early

- tying buffer mutation directly to UI widgets
- letting async services mutate editor state without transactions
- making plugin execution part of the frame path
- creating an unstructured settings sprawl with unclear precedence
- building AI features before the editor command model is solid
- chasing feature parity before the editor core feels excellent
