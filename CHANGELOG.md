# Changelog

## [Unreleased]

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
- Monaco SQL editor
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

- current frontend test suite: must reach 0 failures
- `npm run format:check`: must pass because release preflight enforces it
- exact-SHA frontend + Rust verification
- macOS / Windows / Linux Tauri release builds + artifacts
- packaged/manual runtime smoke

See `plans/07-current-status.md` and `docs/release/0.1.0-readiness.md` for current status.

---

## [0.1.0]

Not released yet. This section will be finalized when the release candidate passes all gates and the tag is created.
