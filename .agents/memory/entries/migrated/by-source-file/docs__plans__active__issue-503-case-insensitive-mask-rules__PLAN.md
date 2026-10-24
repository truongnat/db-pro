---
schema_version: 1
id: "MEM-SRC-5862cee55b1766b3"
title: "Source mirror: docs/plans/active/issue-503-case-insensitive-mask-rules/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-503-case-insensitive-mask-rules/PLAN.md"
source_sha256: "5862cee55b1766b3a40c970bb23418b5ec7319b8f478938ee8fab88a35149253"
mirror_body_sha256: "5862cee55b1766b3a40c970bb23418b5ec7319b8f478938ee8fab88a35149253"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
