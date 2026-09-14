# Changelog

All notable user-facing changes to DB Pro. The shipped application is the native Rust binary
`db-pro-native` (`eframe`/`egui`).

## [0.1.0] - 2026-09-14

DB Pro 0.1.0 is the first release candidate of the native desktop application. Release candidate
SHA `fbf9fdab9100f08f12e29434983f32c18f14ac2f`. The `v0.1.0` tag has not been created; see
`docs/release/0.1.0-readiness.md` for the release decisions and
`docs/release/0.1.0-handoff.md` for the tag/release/rollback procedure.

### Added

- **Native application shell.** Rust `eframe`/`egui` desktop app (`db-pro-native`) with no
  WebView or Node runtime; fonts compiled in, SQLite bundled, no asset downloads at launch.
- **PostgreSQL and SQLite connections.** Create, test, connect, disconnect, reconnect, edit and
  delete; SQLite uses the native OS file picker; previously active connections reconnect on
  startup. Credentials are stored in the OS keyring with an encrypted-file fallback.
- **Connection Explorer.** Schemas, tables and views with targeted refresh; single click opens a
  Data preview, double click promotes it to a permanent tab.
- **Query Editor.** Syntax highlighting, schema-aware completion, current-statement / selection /
  run-all execution from a split Run control, multi-statement results as separate
  `Result 1..N` tabs plus a `Messages` tab, structured diagnostics, SQL formatting, query history
  and local drafts.
- **Table Data Editor.** Virtualized grid with filtering, sorting, pagination, column resize with
  persisted layout, row selection and scoped copy, PK-based staged insert/update/delete with
  Enter-to-stage, pending-change review, patch-style updates that send only changed columns, and
  stable staged revision identities.
- **Schema workbench.** Per-table Data / Columns / Indexes / Relations / Triggers / DDL
  inspection with reconstructed DDL and FK navigation.
- **ER Diagram.** Schema-level canvas with a native graph model, spatial index, three-tier level
  of detail, BFS neighbourhood exploration and a coalescing background layout worker.
- **Agent panel (Preview).** Ask/Edit/Agent modes producing reviewable SQL drafts, with a
  confirmation gate on every database-mutating tool call. Optional provider via
  `GROQ_API_KEY`/`OPENAI_API_KEY`.
- **Export and backup subset.** CSV/TSV export from the result grid; SQLite `VACUUM INTO` backup;
  PostgreSQL backup/restore through `pg_dump`/`pg_restore`.

### Changed

- **The React/TypeScript/Vite frontend and its Tauri WebView host were retired** (2026-09-11) and
  archived under `_archive/frontend/`; the native UI is the only UI. CI, the release pipeline and
  the documentation were switched to `db-pro-native`. The legacy `crates/tauri-app` host is kept
  only for parity comparison and is not shipped.
- **Native visual redesign (waves 1–14):** light theme first, full-window data surfaces,
  context-aware status bar, live query cursor status, keyboard grid focus, ER search recovery,
  composed empty states and compact query overflow actions.
- Explorer sidebar rebuilt as a unified hierarchical navigator with a filter bar and
  views/functions/triggers folders.
- Release packaging moved from Tauri bundles to portable archives + `SHA256SUMS.txt`
  (macOS ARM64 `.tar.gz` with a minimal `DB Pro.app`, Windows x86_64 `.zip`, Linux x86_64
  `.tar.gz`). The compiler is pinned to 1.95.0 through `rust-toolchain.toml`, and the release
  workflow reads and verifies that pin.

### Fixed

- Table single-click opens Data preview instead of structure; double-click promotes Data without
  creating duplicate tabs; "Open Structure" explicitly opens Columns.
- Run's main segment executes directly; the chevron only opens options.
- Shortcut labels follow macOS/Windows/Linux semantics; shortcut and grid focus handling were
  corrected for the native shell.
- Status bar distinguishes reconnecting and error states; startup reconnect behaviour corrected.
- Explorer "Refresh" targets the connection that was clicked and invalidates the backend and
  client metadata caches.
- Row-selection copy no longer uses a document-global keyboard listener, so it stops hijacking
  in-cell editor and input copy.
- Staged writes no longer replay already-successful changes after a partial apply; staged row
  edits no longer use stale full-row snapshots.

### Security & Safety

- Read-only connections are enforced in the backend mutation services, not only in the UI;
  tables without a primary key are read-only in the grid.
- Destructive SQL is classified and confirmed in the canonical action path before execution.
- Table mutations apply atomically per batch: a failure rolls the batch back and keeps all staged
  changes for retry. Concurrent external edits surface a three-way conflict dialog
  (Original / Local / DB Current, Keep Mine / Use Database).
- Connection secrets are kept in the OS keyring and referenced from the metadata database by a
  `secret_ref` indirection rather than being stored inline.
- The release build no longer contains the developer connection preset (label, database name,
  tag, user, password); it is compiled only into debug builds. The component gallery no longer
  renders the developer connection label.
- Release workflow runs with `contents: read`, uses no secrets, and can neither create a GitHub
  Release nor push a tag.

### Known limitations

- Runtime smoke of the 0.1.0 candidate is not complete; V01-01/02/04/05 runtime evidence is
  missing and V01-03 is partial (`docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md`).
- Workspace/tab/settings persistence is not implemented in the native build.
- Query cancellation works for SQLite; PostgreSQL cancellation is capability-gated `Unsupported`.
- Row insert is narrower than a full workflow for complex column types (JSON, array, UUID).
- macOS x86_64 is not built; no installers (`.dmg`, MSI/NSIS, `.deb`/`.rpm`, AppImage) and no
  auto-update.
- Linux credential storage requires a D-Bus Secret Service provider.
- `pg_dump`/`pg_restore` and `ssh` are not bundled and must be on `PATH`.
- CHECK/Unique constraint introspection is unqualified; SSH tunnels are not end-to-end qualified.
- Release artifacts are unsigned: macOS Gatekeeper and Windows SmartScreen will warn.
- The project license is not defined (`R-LICENSE`) — public distribution is blocked until that
  decision is made.
- The Agent panel is Preview and has no recorded live-provider verification run.

---

## Historical (pre-native)

The sections below record the retired frontend/Tauri era. They are kept for history; the
React/Vite frontend was archived on 2026-09-11 and no longer gates release.

### Added / Completed (native release-candidate scope, recorded before 2026-09-14)

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

### Deferred from 0.1.0

- a complete row-insert workflow for complex column types (staged insert ships; JSON/array/UUID
  inputs are incomplete — LIM-003)
- advanced schema mutation / DDL execution workbench
- users / roles workbench
- production/autonomous Agent execution (the Agent panel ships as Preview)
- MCP server
- additional database drivers
- installer formats and code signing/notarization

### Remaining release gates (recorded 2026-09-11, superseded by the 2026-09-14 state above)

- exact-SHA Rust verification (`cargo fmt/check/clippy/test --workspace`)
- `cargo build --release --locked -p db-pro-native`
- macOS / Windows / Linux native release builds + retained artifacts
- packaged/manual runtime smoke on the native app

---

## [Unreleased]

Any change made after the 0.1.0 release candidate cut is recorded here. The 0.1.0 candidate itself
is frozen at `fbf9fdab9100f08f12e29434983f32c18f14ac2f`.

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
  - Development default: in **debug builds only**, a "Xe Lạc Hồng (PostgreSQL)" connection is
    seeded on launch and pre-fills the New connection dialog. Developer convenience, not a
    product default; release builds no longer contain it.
- **UI: React frontend archived; native UI is the development direction (2026-09-11).**
  - `frontend/` moved to `_archive/frontend/`; it is reference material only and is no longer built, tested, or packaged.
  - `bench/` (React Flow / Cytoscape ER-renderer harnesses with vendored React) moved to `_archive/bench/` for the same reason.
  - `crates/ui` (egui), `crates/runtime`, and `crates/native-app` (`db-pro-native`) are the shipped UI stack.
  - CI: the frontend job was removed; the Rust job installs native GL/X11/Wayland headers and builds `db-pro-native`.
  - Release: the pipeline now builds `db-pro-native` for macOS/Windows/Linux instead of Tauri bundles; installer packaging and signing are not implemented yet.
  - `crates/tauri-app` is retained as a legacy transitional host and is marked for removal at cutover.
  - README, AGENTS.md, architecture docs, release docs, and plans were updated to describe native UI as the development direction.

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

See `plans/07-current-status.md`, `docs/release/0.1.0-readiness.md` and
`docs/release/0.1.0-handoff.md` for current status.

## [0.1.0] — historical placeholder

The earlier `## [0.1.0]` section read: *"Not released yet. This section will be finalized when the
release candidate passes all gates and the tag is created."* It is superseded by the dated
`## [0.1.0] - 2026-09-14` section above. The tag is still not created.
