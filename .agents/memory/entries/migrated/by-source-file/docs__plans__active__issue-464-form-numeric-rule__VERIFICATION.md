---
schema_version: 1
id: "MEM-SRC-93d67f77c446b206"
title: "Source mirror: docs/plans/active/issue-464-form-numeric-rule/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-464-form-numeric-rule/VERIFICATION.md"
source_sha256: "93d67f77c446b20653a01ae639f95029bc762d592bc0f46c04530763ce422f6a"
mirror_body_sha256: "93d67f77c446b20653a01ae639f95029bc762d592bc0f46c04530763ce422f6a"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #464 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/464
- Fix: FieldRule::Numeric now checks v.is_finite() after parse::<f64>().
