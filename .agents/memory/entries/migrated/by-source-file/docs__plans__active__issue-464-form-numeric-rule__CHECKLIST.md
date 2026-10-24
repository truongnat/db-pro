---
schema_version: 1
id: "MEM-SRC-a366275fbdb22930"
title: "Source mirror: docs/plans/active/issue-464-form-numeric-rule/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-464-form-numeric-rule/CHECKLIST.md"
source_sha256: "a366275fbdb229307b1b77091aacb31724310325c06b33e6d3d3e36dc466716a"
mirror_body_sha256: "a366275fbdb229307b1b77091aacb31724310325c06b33e6d3d3e36dc466716a"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #464 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/464
- Fix: FieldRule::Numeric now checks v.is_finite() after parse::<f64>().
