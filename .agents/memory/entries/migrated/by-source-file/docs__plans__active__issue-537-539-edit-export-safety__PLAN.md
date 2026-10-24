---
schema_version: 1
id: "MEM-SRC-074878974f664d0d"
title: "Source mirror: docs/plans/active/issue-537-539-edit-export-safety/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-537-539-edit-export-safety/PLAN.md"
source_sha256: "074878974f664d0d7b7ca50b31426a7973f6d669c0cfa927ff2962eab6d41d99"
mirror_body_sha256: "074878974f664d0d7b7ca50b31426a7973f6d669c0cfa927ff2962eab6d41d99"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issues #537 / #538 / #539 — Edit and Export Safety Wave

State: REVIEW
Branch: `main`

## Goal

Close three open P2 correctness tickets from the 2026-10-21 issue batch where
the UI can silently lose or corrupt user data:

- **#537** — a result cell truncated by the runtime mapper (`text > 64 KiB`,
  `bytea > 32 KiB`) is still editable; `Apply` stages the display prefix back
  over the full value.
- **#538** — SQL INSERT/COPY export ignores the connection driver: identifiers
  are always double-quoted and MySQL literals never escape backslashes, so the
  generated SQL fails on MySQL/SQL Server.
- **#539** — the export overwrite confirmation is a `bool`; after changing
  Format or path the armed flag still applies and `Overwrite` replaces a file
  the prompt never named.

Also verify (#540 — Schema tab table-info loading message) is already fixed by
`0e22aae7` and close it with evidence; no code change.

## Evidence / baseline

- #537: `crates/native-app/src/translate_map.rs` caps text/JSON at 65,536 bytes
  and bytea at 32 KiB, appending `…`; nothing downstream could tell a capped
  value from a real one, and `open_cell_inspector` / `begin_data_cell_edit`
  entered the inline editor unconditionally once write access existed.
- #538: `result_grid_export::format_result_sql_insert` hard-coded `"ident"` and
  `''` escaping; `format_result_copy` did the same. The driver string existed
  only for the COPY availability gate.
- #539: `overlay_state.export_overwrite_pending: bool` survived a Format or
  path change; `draw_overwrite_confirmation` displayed the raw typed path while
  the write used `effective_export_path()`.

## Scope

- `crates/ui/src/runtime/runtime_query_types.rs` — shared
  `MAX_CELL_TEXT_BYTES`, `UiCell::is_truncated()`.
- `crates/native-app/src/translate_map.rs` — mapper reads the shared constant.
- `crates/app/src/result_grid/result_grid_edit.rs`,
  `result_grid_inspector_surface_view.rs` — inspector shows the truncation
  notice, hides Export bytes, refuses to open an editor for truncated cells.
- `crates/app/src/table/table_mutation_actions.rs`,
  `table_editor_context.rs` — editor-entry and write-boundary guards.
- `crates/app/src/result_grid/result_grid_export.rs` — dialect-aware INSERT and
  COPY rendering (`DriverDialect::quote_identifier`, driver-specific literal
  escaping, SQL Server `1/0` booleans).
- `crates/app/src/shell/overlay_state.rs`,
  `query_workbench/query_dialogs_view.rs`,
  `query_dialog_surface_view.rs` — overwrite token is the exact effective
  `PathBuf`; any Format/path/Browse change re-arms the prompt for the new
  destination.
- Locales (`en/vi/zh/ja/ko`) — `query.cell_truncated_for_display`.

## Non-goals

- Raising or streaming past the runtime truncation caps (#542 perf is separate).
- A general SQL literal formatter for every dialect (only the export writers).
- Closing the perf tickets #541-#545 or the epics #182/#252/#260.
- Visual redesign of the export dialog or inspector.

## Provider matrix

| Provider | Supported | Required proof |
|---|---|---|
| PostgreSQL | yes | ANSI quoting, `TRUE/FALSE`, COPY quoting — unit tests |
| SQLite | yes | ANSI quoting, `TRUE/FALSE` — unit tests |
| MySQL | yes | backtick identifiers, backslash doubling — unit tests |
| SQL Server | yes | bracket identifiers, `1/0` booleans — unit tests; live run remains BLOCKED |

COPY output stays PostgreSQL-only and is already gated in the dialog.

## Architecture

UI (`DbProApp` result grid / export dialog) → local formatters in
`crates/app` → file write. No command, service, connector or SQL execution path
changes; #538 changes generated text only. #537 blocks a UI-originated write
before it reaches `StagedChange::Update`.

## Acceptance criteria

- [x]#537 A truncated text/JSON/bytes cell never opens an inline editor, the
      inspector warns, Export bytes is unavailable, and `stage_data_cell_edit`
      rejects it defensively.
- [x]#538 Export SQL INSERT/COPY emits dialect-correct identifiers and literals
      for PostgreSQL, SQLite, MySQL and SQL Server.
- [x]#539 `Overwrite` is honored only for the effective path shown in the
      prompt; Format/path/Browse changes re-prompt.
- [x]#540 verified fixed at `0e22aae7` (5 locales, used in Schema tab).
- [x] All workspace gates executed and recorded in VERIFICATION.md.
