# ZVIM

ZVIM is a code editor for developers who want a clean and performant workspace, built with a GPUI-style architecture for rendering and interaction.

## Product Thesis

Most editors are good enough at many things and excellent at very few. ZVIM should be excellent at the workflows that matter most to serious programmers:

- instant startup
- low-latency editing in large files and large repos
- native modal precision without modal friction
- strong built-in Git and language tooling
- deep configuration for users who want total control
- a future community extension ecosystem built on stable, sandboxed APIs

The goal is not to clone Zed. The goal is to build an editor that opens faster than Zed, stays more responsive under load, and feels cleaner and less troublesome in daily use.

## What "Better Than Zed" Means

"Better" needs to be concrete. For ZVIM, it means:

- opening speed beats Zed on comparable hardware
- editing remains responsive under load
- the workspace feels clean and intentional
- built-in workflows have fewer bugs and hassles
- the default workflow is optimized for keyboard-first expert users
- extensions are sandboxed, fast, and designed to scale to a community marketplace
- users can push deep customization when they want total control
- remote and local development feel equally first-class
- crashes or slow language servers do not freeze the editor

## Extension Strategy

ZVIM should not start as an extension-heavy editor. In the beginning, we should focus on a world-class built-in editing experience instead of outsourcing core quality to plugins.

But the long-term plan should absolutely include a VS Code-style community extension model:

- anyone should eventually be able to build and publish extensions
- the extension API should be stable and intentional
- extensions must run outside the frame-critical path
- the marketplace should expand the editor without becoming a crutch for missing core quality

That gives us the upside of an open ecosystem without making the first versions of ZVIM feel hollow or dependency-heavy.

## Configuration Philosophy

ZVIM should ship with excellent defaults, but it should also give power users deep control over how the editor behaves.

That means:

- rich settings for editing, rendering, keymaps, panels, AI behavior, and language tooling
- the ability to tune the editor from conservative defaults to highly customized workflows
- configuration that is structured, inspectable, and predictable instead of scattered magic
- customization that does not require third-party extensions for basic control

The product goal is simple: beginners should get a strong default experience, while experts should feel that almost nothing is artificially locked down.

## V1 Priorities

ZVIM v1 should focus on the built-in experience:

- native modal editing
- clean IDE-style workspace
- tree-first file navigation
- strong built-in Git
- strong built-in language tooling
- deep settings and keymap control
- no AI in v1

See [docs/product-spec.md](/root/zvim/docs/product-spec.md) for the concrete product definition.

## Core Principles

- Performance is a feature, not a cleanup phase.
- The editor core must stay deterministic and testable.
- UI state and editor state should be separate.
- Heavy work must be isolated behind async boundaries.
- Every subsystem should have a clear budget for startup time, frame time, and memory.

## Technical Direction

ZVIM is organized as a Rust workspace with clear subsystem boundaries. The current workspace is intentionally minimal so we can add GPUI cleanly instead of smearing UI concerns across the whole codebase.

Planned stack:

- Rust for core systems
- GPUI for rendering and app shell
- tree-sitter for syntax structure
- LSP for language features
- a local-first command and event system

See [docs/product-spec.md](/root/zvim/docs/product-spec.md) for the product definition, [docs/architecture.md](/root/zvim/docs/architecture.md) for the system design, and [docs/roadmap.md](/root/zvim/docs/roadmap.md) for delivery phases.

## Workspace Layout

- `crates/zvim-app`: executable app shell
- `crates/zvim-core`: shared primitives and domain types
- `crates/zvim-editor`: buffer, cursor, selection, command, and viewport logic
- `crates/zvim-lsp`: language tooling integration boundary

## First Milestone

The first milestone is not "build everything." It is:

1. boot a window quickly
2. open a file
3. edit text with a native modal core
4. render smoothly in a clean workspace shell
5. prove the architecture can absorb Git, LSP, settings, and future extensions without turning brittle

## Next Build Steps

1. define the configuration model and precedence rules
2. wire `zvim-app` to GPUI
3. implement a rope-backed text buffer in `zvim-editor`
4. add a command system with native modal editing support
5. add viewport, tree navigation, and incremental rendering
