# ZVIM Codebase Analysis

## Scope

This review focuses on the current executable Rust workspace as it exists today, not the aspirational product surface described in the README and planning docs.

Current crates:

- `zvim-app`
- `zvim-core`
- `zvim-editor`
- `zvim-lsp`

## Current State

The project is still in a scaffolding phase.

What exists:

- a workspace split into reasonable crate boundaries
- a layered configuration model in `zvim-core`
- a bootstrap path in `zvim-app`
- a minimal GPUI shell that renders a boot report
- placeholder `EditorEngine` and `LspRuntime` types

What does not meaningfully exist yet:

- buffer model
- file opening flow
- editing commands
- modal state
- viewport logic
- tree navigation
- Git integration
- real LSP process management
- async background service boundaries

The codebase is small enough that most of the implementation can be audited directly. The strongest implemented subsystem today is configuration layering. Most other subsystems are placeholders.

## Critical Findings

### 1. GUI startup logic is not portable

File: `crates/zvim-app/src/main.rs`

The app shell only launches when `DISPLAY` or `WAYLAND_DISPLAY` is set. That means macOS and Windows graphical sessions will incorrectly fall back to the headless boot-report path.

Why this matters:

- it makes the current binary effectively Linux-display-env-specific
- it conflicts with the project’s GUI-native direction
- it will create misleading behavior as soon as contributors try to run the app on non-Linux systems

### 2. Window creation failures panic instead of using the error model

File: `crates/zvim-app/src/ui.rs`

`open_window(...)` is wrapped in `.expect("failed to open ZVIM window")`.

Why this matters:

- runtime windowing failures abort the process instead of surfacing as `BootstrapError`
- tests and future startup diagnostics cannot reason about these failures cleanly
- this is a weak foundation for a desktop app that will eventually need predictable recovery and reporting

### 3. Config parse errors lose file-path context

File: `crates/zvim-core/src/config.rs`

`ConfigLoadError::ParseToml` does not retain the source file path. The bootstrap loads multiple layers, but a TOML parse failure does not identify which file was malformed.

Why this matters:

- users cannot quickly fix broken config
- layered settings become harder to debug as the settings surface grows
- this contradicts the architecture goal that invalid settings should fail clearly

### 4. Settings are merged but not validated

File: `crates/zvim-core/src/config.rs`

The settings system accepts structurally valid TOML, but it does not reject semantically invalid values such as `tab_size = 0` or unusable typography numbers.

Why this matters:

- invalid config can silently flow into future rendering and editor logic
- bugs will show up later and farther away from the source of the problem
- the config system is at risk of becoming permissive in exactly the area where the docs call for strictness

### 5. LSP readiness is reported as true without any real runtime

Files:

- `crates/zvim-lsp/src/lib.rs`
- `crates/zvim-app/src/app.rs`

`LspRuntime::is_ready()` always returns `true`, and the boot report exposes that as real state.

Why this matters:

- it teaches the codebase to lie about subsystem health
- future UI and diagnostics may be built on top of false readiness signals
- this makes logs and tests less trustworthy than a blunt placeholder would

## Secondary Concerns

### Docs and implementation are far apart

The architecture and product docs describe a serious editor platform, but the executable code is still at the bootstrap-and-settings stage.

This is not inherently bad in an early project, but it becomes risky if:

- contributors assume major subsystems already exist
- roadmap claims start getting treated as implementation facts
- quality claims are made before the project has a real editor core

### `zvim-editor` and `zvim-lsp` currently provide almost no protective abstraction

The crate boundaries are good in principle, but today they mostly contain placeholders. That means the architecture is still unproven. The real test will be whether the first buffer, command, and async service implementations preserve the intended separation.

### Tooling hygiene is not fully clean

`cargo fmt --check` currently reports formatting diffs. That is small, but it is still worth tightening early while the repository is small.

## What Is Working Well

The codebase does have a few good foundations:

- the workspace split is sensible
- configuration precedence is explicit and easy to read
- nested settings patches are implemented with reasonable clarity
- tests around config merging are better than the rest of the project’s current coverage
- the architecture docs are directionally coherent, even if they are ahead of the implementation

## Recommended Next Steps

The project should not branch into more product surface area yet. The next steps should prove the core architecture with real behavior.

### Phase 1: Tighten the bootstrap and config foundation

1. Make startup platform-aware rather than Linux-display-variable-aware.
2. Replace startup panics with typed errors that flow through `BootstrapError`.
3. Add config validation so invalid values fail with specific, actionable messages.
4. Preserve file-path context for TOML parse errors.
5. Make placeholder subsystem state explicit, for example `Unavailable` or `NotStarted`, instead of reporting success.
6. Add a small CI-quality baseline:
   `cargo fmt --check`
   `cargo test`

Goal:

Make the existing bootstrap trustworthy before more subsystems depend on it.

### Phase 2: Build the first real editor-core slice

1. Implement a real buffer type in `zvim-editor`.
2. Add file loading for a single buffer.
3. Add cursor state and minimal insert-mode editing.
4. Add deterministic tests for buffer mutation and cursor movement.
5. Keep UI rendering as a projection of editor state rather than letting GPUI own text truth.

Goal:

Prove that `zvim-editor` can hold real editor semantics independently of the UI layer.

### Phase 3: Connect the app shell to actual editor state

1. Replace the boot-report-only window with a single-buffer workspace view.
2. Render buffer contents from `EditorEngine`.
3. Wire a narrow command path from input handling to editor mutations.
4. Keep state transitions observable and testable.

Goal:

Move from “bootstrap demo” to “tiny editor” without collapsing architecture boundaries.

### Phase 4: Add one honest background service

1. Pick one subsystem, preferably LSP or file watching.
2. Model its lifecycle explicitly: not started, starting, ready, failed.
3. Pass messages across a boundary instead of allowing hidden shared-state mutation.
4. Add failure-path tests, not only happy-path wiring.

Goal:

Validate the async-service architecture with one real integration before multiplying subsystems.

## Suggested Project Priorities

If the goal is to build a credible editor rather than just a promising scaffold, priorities should be:

1. correctness of bootstrap and settings
2. a real editor core
3. UI projection of real state
4. one background service with honest lifecycle handling
5. only then broader product features like tree navigation, Git, and richer language tooling

## Suggested Non-Priorities For Now

These should probably wait:

- extension architecture work beyond documentation
- remote-workspace foundations beyond keeping boundaries clean
- advanced workspace chrome
- broad feature checklists meant to match mature editors

## Bottom Line

This repository has a decent skeleton and a genuinely solid start on layered configuration, but it is still far from an editor. The best next move is not to widen the roadmap. It is to harden the bootstrap/config base and then prove the architecture with a minimal real editing core.
