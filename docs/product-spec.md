# ZVIM Product Spec

## Product Definition

ZVIM is a code editor for developers who want a clean and performant workspace.

Its competitive goal is not just to be feature-rich. It is to feel faster, cleaner, and less frustrating than Zed in the workflows that matter every day.

## Success Criteria

ZVIM should aim to beat Zed in three practical areas:

- opening speed
- responsiveness during editing
- sustained performance as projects and features scale

ZVIM should also aim to feel better than Zed in product quality:

- cleaner user experience
- fewer bugs
- fewer rough edges and hassles
- stronger sense of control for expert users

## Target User

ZVIM is for developers who:

- care about startup speed and low-latency interaction
- want a clean workspace without visual noise
- prefer strong built-in workflows over immediate plugin dependency
- want deep customization when they choose to use it
- value modal precision for editing

## Product Positioning

ZVIM should feel like:

- a serious daily-driver editor
- a clean high-performance workspace
- a GUI-native editor with native modal editing
- a tool that stays predictable under load

ZVIM should not feel like:

- a plugin shell waiting for the ecosystem to make it useful
- an AI-first product
- a cluttered IDE with weak defaults

## Chosen Product Shape

The current product choices are:

- native modal editing from day one
- IDE-style workspace with visible panels and controls
- tree-first file navigation
- strong built-in Git support
- strong built-in language tooling
- no AI in v1
- deep configuration and settings control
- architecture prepared for a future VS Code-style extension ecosystem
- remote-ready foundations in the architecture
- overall identity: the cleanest high-performance coding workspace

## Core V1 Features

These are the built-in features that should define early ZVIM.

### 1. Native Modal Editing

This is not an afterthought compatibility layer.

V1 should include:

- normal and insert modes
- motions and text objects
- multi-cursor aware command handling
- repeatable command model
- reliable undo and redo

### 2. Clean IDE-Style Workspace

ZVIM should support a full workspace experience while staying visually disciplined.

V1 should include:

- file tree
- tabs or buffers list
- command palette
- searchable navigation
- docked panels that do not feel noisy or heavy

### 3. Tree-First Navigation

The file tree should be first-class, not bolted on.

V1 should include:

- fast project tree
- quick open
- recent files
- symbol navigation
- workspace switching

### 4. Strong Built-In Git

Git should feel native and low-friction.

V1 should include:

- gutter diff indicators
- inline diff visibility
- hunk stage and revert
- file stage and unstage
- blame on demand
- branch awareness

### 5. Strong Built-In Language Features

Language tooling should be part of the editor's core value, not a weak placeholder.

V1 should include:

- diagnostics
- hover
- completion
- go-to-definition
- rename
- code actions

### 6. Deep Configuration

Users who want total control should be able to shape the editor to their workflow.

V1 should include:

- user settings
- workspace settings
- keymap customization
- appearance and layout controls
- editing behavior controls
- language-specific settings
- Git and extension-related settings boundaries

### 7. Performance As A Product Feature

Performance should be observable and defended.

V1 should include:

- fast startup target
- incremental rendering
- isolation between UI and heavy background tasks
- graceful behavior on large repos and large files

## Explicit Non-Goals For V1

These are intentionally not first-wave priorities:

- AI chat or AI coding features
- a large extension marketplace at launch
- heavy collaboration features
- trying to match every Zed feature before the core feels excellent

## Extension Strategy

ZVIM should prepare now for a future community extension ecosystem similar in spirit to VS Code.

But the launch philosophy is:

- ship strong built-ins first
- define stable APIs second
- open the ecosystem after the core is solid

Extensions should be:

- sandboxed
- explicit in permissions
- kept off the critical rendering path
- unable to destabilize the editor core

## Configuration Strategy

ZVIM should support broad user control without becoming chaotic.

The settings model should be:

- layered across default, user, workspace, and project scopes
- easy to inspect
- predictable in precedence
- live-reloadable where practical
- strict about invalid configuration

## Remote Foundation Strategy

ZVIM does not need full remote UX in v1, but the architecture should leave room for:

- remote workspaces
- remote indexing and language tooling
- low-friction reconnect behavior
- local and remote workflows that feel consistent

## Product Metrics

The product should be measured against concrete goals.

Initial targets:

- visible startup faster than Zed on comparable hardware
- keypress-to-paint budget under 8 ms in normal workflows
- no UI freezes from LSP, parsing, or Git work
- large repo navigation that remains smooth and predictable

## Delivery Order

The most effective order is:

1. build the editor core
2. build the config system
3. build the GPUI workspace shell
4. build tree navigation and command palette
5. build Git integration
6. build language tooling
7. prepare extension boundaries

## Standard For Release

ZVIM should not claim to compete with Zed until all of the following are true:

- startup feels instant
- editing feels crisp every day
- the workspace looks clean and intentional
- core Git and language workflows are reliable
- users can tune the editor deeply without fighting it
- the product feels polished rather than ambitious-but-rough
