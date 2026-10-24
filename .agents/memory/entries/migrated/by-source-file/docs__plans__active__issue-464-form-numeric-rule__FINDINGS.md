---
schema_version: 1
id: "MEM-SRC-d6d0e18dfb2ff19e"
title: "Source mirror: docs/plans/active/issue-464-form-numeric-rule/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-464-form-numeric-rule/FINDINGS.md"
source_sha256: "d6d0e18dfb2ff19e5b3a31bbc84617051e5ac48a888ad5976f5bb2291ae0c796"
mirror_body_sha256: "d6d0e18dfb2ff19e5b3a31bbc84617051e5ac48a888ad5976f5bb2291ae0c796"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #464 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/464
- Fix: FieldRule::Numeric now checks v.is_finite() after parse::<f64>().
