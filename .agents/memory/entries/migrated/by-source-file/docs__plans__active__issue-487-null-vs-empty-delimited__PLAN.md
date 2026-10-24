---
schema_version: 1
id: "MEM-SRC-5b1a51658fc301f6"
title: "Source mirror: docs/plans/active/issue-487-null-vs-empty-delimited/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-487-null-vs-empty-delimited/PLAN.md"
source_sha256: "5b1a51658fc301f699564045310f2796a3a62354778005e363e70e0cb24b9ee9"
mirror_body_sha256: "5b1a51658fc301f699564045310f2796a3a62354778005e363e70e0cb24b9ee9"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #487 — NULL vs empty string in delimited export/copy

## Goal
`format_cell_delimited` encoded `UiCell::Null` and `UiCell::Text("")` identically
(empty field) so a TSV/CSV paste or export lost the NULL vs `''` distinction, and
`copy_cell_at` reported "nothing to copy" for an empty text cell.

## Scope
- `result_grid_export.rs`: `UiCell::Null` → `\N` (Postgres `\copy` sentinel) in
  `format_cell_delimited`; empty text stays an empty field. Documented that a
  literal `\N` text value collides with the sentinel (RFC-4180 quoting kept).
- `result_grid_clipboard.rs`: `copy_cell_at` copies `\N` for NULL, and copies
  empty string as an empty `CopyText` command (status "Cell copied" — egui-winit
  may ignore empty clipboard writes, platform limitation).

## Non-goals
- No backslash-escaping of text fields (would break CSV for spreadsheets).
- No import-side parser change; the file transfer path is a separate pipeline.

## Acceptance
- `format_result_delimited` TSV emits `\N\t` for `[Null, Text("")]`.
- `copy_cell_at` on `Text("")` issues `CopyText("")` + "Cell copied"; NULL → `\N`.
- `cargo test -p db-pro-app` + `cargo check -p db-pro-app`.
