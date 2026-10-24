---
schema_version: 1
id: "MEM-PLAN-issue-502-mask-null-literal-bypa-e690e3646e5e"
title: "Plan archive: Issue 502 Mask Null Literal Bypass"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-502-mask-null-literal-bypass]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "e690e3646e5e4a4f8cdb86b3d7f83b21e6b822d46aab5d231f503341ce5c02c8"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-502-mask-null-literal-bypass/PLAN.md"
  - "docs/plans/active/issue-502-mask-null-literal-bypass/FINDINGS.md"
  - "docs/plans/active/issue-502-mask-null-literal-bypass/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-502-mask-null-literal-bypass/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #502 — Literal "null" bypasses masking

## Root cause
`mask_cell(value: &str, …)` treated the *text* "null" (any ASCII case) as SQL
NULL and returned `"NULL"` before applying the column rule. A `&str` cannot
distinguish a real SQL NULL from the literal string "null", so:

- a cell containing the text "null" skipped masking entirely, and
- the emitted "NULL" was indistinguishable from a real NULL in the export.

## Fix
- `mask_cell(value: Option<&str>, rule, keyed, key_material) -> String`
- `None` (SQL NULL) is preserved verbatim as `"NULL"` for *every* rule —
  masking must never rewrite a real NULL into masked-looking content.
- The `eq_ignore_ascii_case("null")` early return and the dead `PreserveNull`
  empty-check were removed; `Some("null")` is ordinary data and gets masked.
- `mask_row` now takes `cells: &[Option<&str>]` so NULL-typed inputs can reach
  `mask_cell`; unmatched `None` cells emit `"NULL"`.
- `preview_masking` / `mask_transfer_batch` wrap their `String` cells in `Some`.

## Out of scope
`TransferRow.cells: Vec<String>` still cannot represent a real SQL NULL — the
transfer path stringifies values upstream. Changing that type would touch the
transfer pipeline; the masking API is now NULL-capable for callers that have
typed cells (e.g. `UiCell::Null → None`).


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

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

