---
schema_version: 1
id: "MEM-PLAN-issue-503-case-insensitive-mask--acc59eeb04bc"
title: "Plan archive: Issue 503 Case Insensitive Mask Rules"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-503-case-insensitive-mask-rules]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "acc59eeb04bc52a181f8a6e38d4f1ff067158da7b7cc857cd55f06ebebb0c826"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-503-case-insensitive-mask-rules/PLAN.md"
  - "docs/plans/active/issue-503-case-insensitive-mask-rules/FINDINGS.md"
  - "docs/plans/active/issue-503-case-insensitive-mask-rules/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-503-case-insensitive-mask-rules/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #503 — Mask rule lookup is case-sensitive and silent

## Root cause
`mask_row` matched profile rules to headers with `c.column == col`. A DB
header `Email` against a profile rule `email` (suggest already lowercases)
matched nothing, so the column exported raw with no signal.

## Fix
- Rule lookup uses `eq_ignore_ascii_case` in `mask_row` and in the new
  `unmatched_mask_rules` helper.
- Visibility: `unmatched_mask_rules(headers, profile)` returns the profile
  columns matching no header; `preview_masking` stores them on
  `MaskingPreview.unmatched_rules` (serde-defaulted) and appends
  "· N rule(s) match no column: …" to `preview.message`, which the UI already
  renders.
- `mask_transfer_batch` emits `tracing::warn!` listing unmatched rules so the
  export path is not silent either.


## Source: `FINDINGS.md`

# Issue #503 — Findings

- `suggest_sensitive_columns` lowercases for detection but returns the
  original header names; users can also type any casing into the columns CSV,
  so strict equality could miss real columns.
- Unmatched rules (not unmatched headers) are the dangerous direction: a rule
  that matches nothing means the user *intended* masking that never happens.
  Headers without rules are ordinary pass-through and need no warning.
- `MaskingPreview.message` is already rendered by
  `transfer_activity_surface_view::draw_masking_result`, so appending the
  unmatched-rule names there reaches the user with no UI change.


## Source: `VERIFICATION.md`

# Issue #503 — Verification

- `cargo check -p db-pro-core` — clean.
- `cargo test -p db-pro-core` — 502 passed, 0 failed, including:
  - `row_rule_matching_is_case_insensitive`: header `Email` masked by profile
    rule `email` → `"***"`.
  - `unmatched_rules_are_surfaced`: `unmatched_mask_rules` returns `["ssn"]`
    for headers `["id","email"]`; `preview.unmatched_rules` and
    `preview.message` both name the missed column; the matching `Email` rule
    still applied (`"[masked]"`).
- `cargo test -p db-pro-app masking` — 2 passed.
- `cargo check --workspace` — clean.

