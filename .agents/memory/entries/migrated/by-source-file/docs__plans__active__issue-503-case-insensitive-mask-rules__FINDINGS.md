---
schema_version: 1
id: "MEM-SRC-6becba7f4e3e0a12"
title: "Source mirror: docs/plans/active/issue-503-case-insensitive-mask-rules/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-503-case-insensitive-mask-rules/FINDINGS.md"
source_sha256: "6becba7f4e3e0a12fd13c43a72b015f30255c74e1fe0e9175fae43bc056ef4bd"
mirror_body_sha256: "6becba7f4e3e0a12fd13c43a72b015f30255c74e1fe0e9175fae43bc056ef4bd"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
