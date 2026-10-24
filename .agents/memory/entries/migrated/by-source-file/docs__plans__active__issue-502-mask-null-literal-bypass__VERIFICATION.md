---
schema_version: 1
id: "MEM-SRC-bfe77a0b4470545a"
title: "Source mirror: docs/plans/active/issue-502-mask-null-literal-bypass/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-502-mask-null-literal-bypass/VERIFICATION.md"
source_sha256: "bfe77a0b4470545a9fd46880f3b0d0ad8c852274dabd6f53baecd3eab01b7f84"
mirror_body_sha256: "bfe77a0b4470545a9fd46880f3b0d0ad8c852274dabd6f53baecd3eab01b7f84"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #502 — Verification

- `cargo check -p db-pro-core` — clean.
- `cargo test -p db-pro-core` — 502 passed, 0 failed, including:
  - `literal_null_text_is_masked_like_data`: `Some("null")`/`Some("NULL")` →
    `"***"` under Redact; `Some("null")` → `"null"` verbatim under
    PreserveNull.
  - `real_null_is_preserved_for_every_rule`: `mask_cell(None, …)` → `"NULL"`
    for all six rules.
  - `row_passes_real_null_through`: `None` cell → `"NULL"` in `mask_row`.
- `cargo test -p db-pro-app masking` — 2 passed (`build_preview` call sites
  unchanged; `String` cells wrapped internally).
- `cargo check --workspace` — clean.
