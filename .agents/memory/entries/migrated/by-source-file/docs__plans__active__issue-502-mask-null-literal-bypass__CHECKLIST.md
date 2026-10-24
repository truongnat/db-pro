---
schema_version: 1
id: "MEM-SRC-255347799ac97228"
title: "Source mirror: docs/plans/active/issue-502-mask-null-literal-bypass/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-502-mask-null-literal-bypass/CHECKLIST.md"
source_sha256: "255347799ac97228bf9d6eef90bf21d00a101ee1dc6d8c4b7b5ac3d20977b7c9"
mirror_body_sha256: "255347799ac97228bf9d6eef90bf21d00a101ee1dc6d8c4b7b5ac3d20977b7c9"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #502 — Checklist

- [x] `mask_cell(Option<&str>)`; `None` → `"NULL"` for all rules
- [x] Removed `eq_ignore_ascii_case("null")` bypass
- [x] `mask_row` accepts `&[Option<&str>]`
- [x] Updated `preview_masking` and `mask_transfer_batch` call sites
- [x] Updated app callers (`build_preview`, `masked_csv_export_outcome`) — no signature change needed
- [x] Tests: literal "null" masked; `None` preserved for every rule
- [x] `cargo test -p db-pro-core masking` — 9 passed
