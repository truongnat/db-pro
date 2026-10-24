---
schema_version: 1
id: "MEM-SRC-ec4bf60ac9e4114c"
title: "Source mirror: docs/plans/active/issue-487-null-vs-empty-delimited/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-487-null-vs-empty-delimited/VERIFICATION.md"
source_sha256: "ec4bf60ac9e4114c662d967c783c3ed3a364d17bd8463285f9b8a57609167ce6"
mirror_body_sha256: "ec4bf60ac9e4114c662d967c783c3ed3a364d17bd8463285f9b8a57609167ce6"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification — #487

- `cargo check -p db-pro-app`: clean.
- `cargo test -p db-pro-app --lib`: 591 passed, 0 failed.
  - `result_grid::tests::delimited_cells_distinguish_null_from_empty_text` — new.
  - `result_grid::tests::copy_cell_copies_empty_text_and_null_sentinel` — new.
  - `result_grid::tests::copied_row_text_distinguishes_null_from_empty_text` — new.
  - `app::result_grid_view::record_tests::record_click_copies_null_and_empty_values` — updated.
  - `app::tests::test_format_cell_csv_and_cell_to_json`,
    `app::tests::test_export_result_writes_escaped_delimited_text` — updated to `\N`.
