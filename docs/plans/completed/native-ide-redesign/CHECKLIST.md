# Native IDE Redesign Checklist

## P0 — Workbench foundation

- [x] Stable topbar, activity rail, explorer, workspace, optional Agent panel, bottom output and status bar.
- [x] Resizable panel widths and visibility persist through eframe storage.
- [x] Shared controls expose visible focus and standard text editing behavior.
- [x] Cmd/Ctrl+K opens the palette; editor and input shortcuts remain native.

## P1 — Explorer

- [x] Connection/database/schema/object hierarchy is compact and bounded.
- [x] Table, view and function context actions are available without toolbar duplication.
- [x] Loading, empty, error and large-schema states remain recoverable.

## P2 — Query workspace

- [x] Query documents remain independent tabs with close/recovery behavior.
- [x] Results, messages, explain/query-plan and history are separate output views.
- [x] Selection execution, format, diagnostics and connection/schema context remain discoverable.

## P3 — Data grid

- [x] Cell edits stage locally and show dirty state.
- [x] Apply and discard are explicit; read-only/no-PK safety remains enforced.
- [x] Sort/filter/resize/virtualization/copy/paste behavior is retained.

## P4–P7 — Capability depth

- [x] Table workspace exposes structure/indexes/relations/constraints/DDL/dependencies.
- [x] Command palette covers real actions and Quick Open uses the same search model.
- [x] Agent receives database context and produces reviewable SQL drafts with safe apply/run paths.
- [x] Exposed backup/restore, Explain, routine navigation and schema-map paths use provider capabilities; result export is provider-neutral because it writes an already loaded result.
- [x] Future user/session/lock/sequence/type surfaces remain explicitly unsupported until their runtime commands and provider evidence exist.

## Verification

- [x] Unit/regression tests for state transitions and request routing.
- [x] Rust quality gates pass.
- [x] PostgreSQL runtime evidence recorded separately from SQLite.
- [x] Native UI walkthrough evidence recorded separately for PostgreSQL and SQLite.
