---
schema_version: 1
id: "MEM-SRC-3adb13a9a1b19efc"
title: "Source mirror: docs/plans/active/issue-502-mask-null-literal-bypass/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-502-mask-null-literal-bypass/FINDINGS.md"
source_sha256: "3adb13a9a1b19efcaf4bff99f801d2870afe0fcb6e3ad3e1e08fe1b38edfc742"
mirror_body_sha256: "3adb13a9a1b19efcaf4bff99f801d2870afe0fcb6e3ad3e1e08fe1b38edfc742"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #502 — Findings

- `mask_cell` had *three* NULL-related shortcuts: a top-level
  `eq_ignore_ascii_case("null")` bail-out for all rules, a redundant
  `PreserveNull` empty-string check, and a nested `PreserveNull` "null" check.
- All production callers feed `Vec<String>` cells (`preview_masking` sample
  rows in `app::tools::masking::build_preview`, `mask_transfer_batch` over
  `TransferRow.cells` in `transfer_harness_view.rs`). No caller held `UiCell`,
  so `mask_row` accepts `&[Option<&str>]` to carry NULL-ness without churning
  `TransferRow`.
- `PreserveNull` on non-null values already returned the value verbatim, so
  removing the special-cases leaves that rule's semantics unchanged.
