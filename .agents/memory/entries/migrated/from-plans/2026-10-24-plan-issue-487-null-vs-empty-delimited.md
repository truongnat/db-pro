---
schema_version: 1
id: "MEM-PLAN-issue-487-null-vs-empty-delimite-926d78fe5f5a"
title: "Plan archive: Issue 487 Null Vs Empty Delimited"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-487-null-vs-empty-delimited]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "926d78fe5f5ad908804769933520a8c6f7b23fa84a8882da235a099a48b7ac2f"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-487-null-vs-empty-delimited/PLAN.md"
  - "docs/plans/active/issue-487-null-vs-empty-delimited/FINDINGS.md"
  - "docs/plans/active/issue-487-null-vs-empty-delimited/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-487-null-vs-empty-delimited/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Findings — #487

- `format_cell_delimited` (`result_grid_export.rs`) returned `String::new()` for
  `UiCell::Null` — identical to `Text("")` → lossy paste/export. Fixed to emit
  `\N`, the Postgres `\copy` NULL sentinel; `format_cell_copy_field` already used
  `\N` for the `COPY`-format export path.
- `copy_cell_at` (`result_grid_clipboard.rs`) early-returned "Empty value:
  nothing to copy" whenever `cell_text` was empty. `copy_cell_value` already
  returns `Some(Text(""))` fine — the gate lived in the copy action. NULL now
  copies as `\N`; empty text copies as `""`.
- `copied_row_text` picks up the sentinel for free via `format_cell_delimited`.
- Tests updated: `app_tests.rs::test_format_cell_csv_and_cell_to_json`,
  `result_grid_record_tests.rs::record_click_copies_null_and_empty_values`
  (renamed; record-view clicks go through `copy_cell_at`).
- Import side: `csv_cell_from_display(.., treat_empty_as_null_token)` is a
  provider-side display toggle, not a `\N` parser — out of scope.


## Source: `VERIFICATION.md`

# Verification — #487

- `cargo check -p db-pro-app`: clean.
- `cargo test -p db-pro-app --lib`: 591 passed, 0 failed.
  - `result_grid::tests::delimited_cells_distinguish_null_from_empty_text` — new.
  - `result_grid::tests::copy_cell_copies_empty_text_and_null_sentinel` — new.
  - `result_grid::tests::copied_row_text_distinguishes_null_from_empty_text` — new.
  - `app::result_grid_view::record_tests::record_click_copies_null_and_empty_values` — updated.
  - `app::tests::test_format_cell_csv_and_cell_to_json`,
    `app::tests::test_export_result_writes_escaped_delimited_text` — updated to `\N`.

