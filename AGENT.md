# ZVIM Agent Operations Guide

## Project Overview

ZVIM is a high-performance code editor built with Rust and GPUI, targeting competitive performance against Zed with better startup speed and responsiveness.

**Product Goals:**
- Instant startup (< 120ms cold start)
- Low-latency editing in large files/repos
- Native modal precision without friction
- Strong built-in Git and language tooling
- Deep configuration for power users
- Future sandboxes extension ecosystem

**Stack:** Rust + GPUI + tree-sitter + LSP

---

## Workspace Structure

```
/home/falloficaruss/zvim/
├── Cargo.toml          # Workspace root
├── crates/
│   ├── zvim-app/     # GPUI app shell, window management
│   ├── zvim-core/    # Shared types, config system, latency budgets
│   ├── zvim-editor/  # Buffer, cursor, selection, commands, viewport
│   └── zvim-lsp/     # LSP client runtime
└── docs/
    ├── architecture.md
    ├── product-spec.md
    ├── roadmap.md
    └── codebase-analysis.md
```

**Crate Dependencies:**
- `zvim-app` → `gpui` + `zvim-core` + `zvim-editor` + `zvim-lsp`
- `zvim-editor` → `zvim-core`
- `zvim-core` → `serde`, `toml`
- `zvim-lsp` → (none yet)

---

## Current Implementation State

### Working Subsystems
- Workspace crate boundaries
- Layered configuration model (default → user → workspace → project)
- Bootstrap path with GPUI window creation

### Placeholders / Not Yet Implemented
- Buffer model (needs rope-backed implementation)
- File opening flow
- Editing commands and modal state
- Viewport logic
- Tree navigation
- Git integration
- Real LSP process management
- Async background service boundaries

### Known Issues (from codebase-analysis.md)
1. GUI startup only works on Linux with `DISPLAY`/`WAYLAND_DISPLAY`
2. Window creation failures panic with `.expect()`
3. Config parse errors don't retain file path context
4. Settings are merged but not validated (e.g., `tab_size = 0` not rejected)
5. LSP runtime reports `is_ready() = true` with no real implementation

---

## Phase Priorities

### Current Phase: Foundation → Fast Editor Core
Per roadmap.md, the active build targets:

1. **rope-backed buffer** - performant text storage
2. **cursor and selection engine** - multi-cursor support
3. **insert and normal modes** - native modal editing
4. **undo/redo transactions** - history model
5. **viewport abstraction** - visible region management
6. **command model** - designed for native modal editing

### Delivery Order
1. Build editing engine
2. Build config system
3. Attach GPUI to engine
4. Add tree-first workspace navigation
5. Add built-in Git
6. Add language services
7. Add advanced workflows
8. Deepen differentiation
9. Open community extension platform

---

## Development Commands

```bash
# Build all crates
cargo build

# Run the app
cargo run -p zvim-app

# Run tests
cargo test

# Format check
cargo fmt --check

# Lint
cargo clippy

# Workspace lint/format
cargo fmt --check && cargo clippy --all-targets
```

**Note:** Per codebase-analysis.md, `cargo fmt --check` currently reports diffs.

---

## Code Conventions

### Architecture Principles
- UI observes domain state; commands flow into editor engine
- Editor engine must stay deterministic and testable
- Background services run async, isolated from frame-critical work
- Heavy computations happen outside render loops
- Every subsystem has a budget for startup time, frame time, memory

### Subsystem Boundaries
- **zvim-app**: GPUI rendering, window/panel management, input routing
- **zvim-editor**: Buffer truth, cursors, selections, modal commands
- **zvim-core**: Shared types, config, command envelopes, events
- **zvim-lsp**: LSP process management, diagnostics, completion

### Error Handling
- Use typed `BootstrapError` for startup failures (not panics)
- Subsystem lifecycle states: `NotStarted` → `Starting` → `Ready`/Failed
- Config validation must reject semantically invalid values with clear messages

### Testing Requirements
- Buffer mutation tests
- Cursor movement tests
- Config merge/validation tests
- Happy path + failure path for async services

---

## V1 Feature Scope (Explicit)

### Must Have
- Native modal editing (normal + insert modes)
- Clean IDE-style workspace (file tree, tabs, command palette)
- Tree-first navigation
- Strong built-in Git (gutter diffs, stage/revert, blame)
- Strong built-in language tooling (diagnostics, hover, completion, go-to-def, rename)
- Deep configuration
- Performance as feature

### Must NOT Have (V1)
- AI chat or AI coding features
- Large extension marketplace
- Heavy collaboration features

---

## Key Files Reference

| File | Purpose |
|------|---------|
| `crates/zvim-app/src/main.rs` | App bootstrap, entry point |
| `crates/zvim-app/src/app.rs` | App state management |
| `crates/zvim-app/src/ui.rs` | GPUI view composition |
| `crates/zvim-core/src/config.rs` | Layered config model |
| `crates/zvim-core/src/lib.rs` | Core shared types |
| `crates/zvim-editor/src/lib.rs` | Buffer, cursor, commands |
| `crates/zvim-lsp/src/lib.rs` | LSP runtime boundary |
| `docs/architecture.md` | System design goals |
| `docs/product-spec.md` | Product definition |
| `docs/roadmap.md` | Build phases |
| `docs/codebase-analysis.md` | Current state + issues |

---

## Operational Notes

- **GPUI integration**: Use GPUI as renderer only; editor logic stays in `zvim-editor`
- **Config precedence**: default → user → workspace → project (documented in `zvim-core`)
- **Performance budgets**: cold start < 120ms, keypress-to-paint < 8ms, command dispatch < 4ms
- **Extension strategy**: Built-ins first, stable APIs second, ecosystem third
- **No AI in V1**: Explicit non-goal per product spec