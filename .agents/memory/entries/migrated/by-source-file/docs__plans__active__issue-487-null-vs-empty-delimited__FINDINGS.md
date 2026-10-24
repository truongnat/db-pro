---
schema_version: 1
id: "MEM-SRC-5ea2d32625824e59"
title: "Source mirror: docs/plans/active/issue-487-null-vs-empty-delimited/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-487-null-vs-empty-delimited/FINDINGS.md"
source_sha256: "5ea2d32625824e597a594db21ae6106b416f30a621d43777eff61ae16e238197"
mirror_body_sha256: "5ea2d32625824e597a594db21ae6106b416f30a621d43777eff61ae16e238197"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
