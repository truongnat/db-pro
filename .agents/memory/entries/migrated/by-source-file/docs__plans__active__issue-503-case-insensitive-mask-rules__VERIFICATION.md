---
schema_version: 1
id: "MEM-SRC-51afb0723878c8bb"
title: "Source mirror: docs/plans/active/issue-503-case-insensitive-mask-rules/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-503-case-insensitive-mask-rules/VERIFICATION.md"
source_sha256: "51afb0723878c8bb2bde718fb42d262802605d05bd23c8a2679ec34d1b801e10"
mirror_body_sha256: "51afb0723878c8bb2bde718fb42d262802605d05bd23c8a2679ec34d1b801e10"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
