---
schema_version: 1
id: "MEM-SRC-0707a2c653c05121"
title: "Source mirror: docs/plans/active/issue-503-case-insensitive-mask-rules/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-503-case-insensitive-mask-rules/CHECKLIST.md"
source_sha256: "0707a2c653c05121eb00f9cf0bb0e538883e9a503572b3f945dc21d335d22ef7"
mirror_body_sha256: "0707a2c653c05121eb00f9cf0bb0e538883e9a503572b3f945dc21d335d22ef7"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #503 — Checklist

- [x] `mask_row` rule lookup uses `eq_ignore_ascii_case`
- [x] `unmatched_mask_rules(headers, profile)` helper
- [x] `MaskingPreview.unmatched_rules` + message suffix surfaced in preview
- [x] `tracing::warn!` on unmatched rules in `mask_transfer_batch`
- [x] Tests: `Email` header masked by `email` rule; unmatched rule listed
- [x] `cargo test -p db-pro-core masking` — 9 passed
