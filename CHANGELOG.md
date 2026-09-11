# Changelog

## [Unreleased]

### Changed

- **Native UI: visual redesign and DBeaver/Codex sidebar consolidated into main (2026-09-11).**
  - Native visual redesign (waves 1–14): Codex-aligned theme tokens, full-window data surfaces,
    context-aware status bar, live query cursor status, keyboard grid focus, ER search recovery,
    staged-grid interaction correctness, composed empty states, compact query overflow actions.
  - Explorer sidebar rebuilt as a unified DBeaver/Codex hierarchical navigator with a filter bar,
    views/functions/triggers folders and a Codex-style tree row.
  - Light theme is now the default (`THEME_STORAGE_VERSION = "light-first-v1"`); window opens
    maximized.
  - New shared components (spinner, progress bar, skeleton, switch, segmented control, kbd chip,
    tag chip, status dot, toast) added to `crates/ui` and exported from `lib.rs`.
  - Development default: a "Xe Lạc Hồng (PostgreSQL)" connection is seeded on launch and
    pre-fills the New connection dialog. Developer convenience, not a product default.
- **UI: React frontend archived; native UI is the development direction (2026-09-11).**
  - `frontend/` moved to `_archive/frontend/`; it is reference material only and is no longer built, tested, or packaged.
  - `bench/` (React Flow / Cytoscape ER-renderer harnesses with vendored React) moved to `_archive/bench/` for the same reason.
  - `crates/ui` (egui), `crates/runtime`, and `crates/native-app` (`db-pro-native`) are the shipped UI stack.
  - CI: the frontend job was removed; the Rust job installs native GL/X11/Wayland headers and builds `db-pro-native`.
  - Release: the pipeline now builds `db-pro-native` for macOS/Windows/Linux instead of Tauri bundles; installer packaging and signing are not implemented yet.
  - `crates/tauri-app` is retained as a legacy transitional host and is marked for removal at cutover.
  - README, AGENTS.md, architecture docs, release docs, and plans were updated to describe native UI as the development direction.

### 0.1.0 Release Candidate

DB Pro 0.1.0 is currently in release-candidate verification. The intended release scope is largely implemented, but final automated gates, cross-platform artifacts, and manual runtime smoke are still pending.

### Added / Completed

**Database support**
- PostgreSQL connection lifecycle
- SQLite file connections + native Browse wiring
- startup reconnect source flow

**Explorer / DB Object**
- schemas, tables, views
- targeted connection metadata refresh
- Data-first table/view navigation
- Data / Columns / Indexes / Relations / DDL inspection

**Query workbench**
- Native SQL editor
- current-statement execution
- selection execution
- run-all / multi-statement execution
- split Run button
- Explain
- SQL format
- query history
- result grid
- Stop/Escape cancellation source flow

**Workspace / navigation**
- query and DB Object tabs
- preview/pin/close/navigation
- compact inactive pinned tabs
- collision-aware resource titles
- orphan-tab recovery
- Quick Open and Command Palette

**Data Grid**
- virtualization
- filtering, sorting, pagination
- column resize/layout persistence
- row selection/copy
- PK-based staged update/delete
- patch-style row updates (only changed columns sent)
- same-row multi-cell patch composition
- stable staged revision IDs and in-flight safety
- partial-success/failure cleanup by exact revision ID
- apply confirmation for destructive staged changes

**Architecture / safety**
- canonical Action Platform runtime
- confirmation lifecycle
- cancellation identity
- read-only backend policy
- secret/keyring infrastructure

### Fixed during release UX closure

- table single-click now opens Data preview rather than structure/config
- table/view double-click promotes Data without duplicate tabs
- Open Structure explicitly opens Columns
- Run main segment executes directly; chevron only opens options
- shortcut labels aligned across macOS/Windows/Linux semantics
- status bar distinguishes reconnecting/error states
- startup reconnect behavior corrected
- SQLite dialog plugin registered on Rust side with minimal capability
- Explorer Refresh targets the clicked connection and invalidates backend/client metadata caches
- row selection copy no longer uses a document-global keyboard listener
- staged writes no longer replay already-successful changes after partial apply
- staged row edits no longer use stale full-row snapshots
- platform-specific Rollup native binary removed from root dependencies; lockfile regenerated

### Explicitly deferred from 0.1.0

- complete row insertion workflow
- advanced schema mutation / DDL execution workbench
- users / roles workbench
- production Agent execution
- MCP server
- additional database drivers

### Known release limitations

- SSH tunnel plumbing is not yet end-to-end qualified on all release targets.
- Release artifacts are unsigned unless signing/notarization is added before distribution.
- Tables without primary keys are read-only in the Data Grid.
- Agent panel is Preview only.
- Project license is not yet defined.

### Remaining release gates

- exact-SHA Rust verification (`cargo fmt/check/clippy/test --workspace`)
- `cargo build --release --locked -p db-pro-native`
- macOS / Windows / Linux native release builds + retained artifacts
- installer/packaging format decision for the native binary (DMG, MSI/NSIS, DEB/RPM/AppImage)
- packaged/manual runtime smoke on the native app

See `plans/07-current-status.md` and `docs/release/0.1.0-readiness.md` for current status.

---

## [0.1.0]

Not released yet. This section will be finalized when the release candidate passes all gates and the tag is created.
