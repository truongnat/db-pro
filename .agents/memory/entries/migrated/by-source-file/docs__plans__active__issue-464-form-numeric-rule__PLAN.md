---
schema_version: 1
id: "MEM-SRC-01490cd1b1fac185"
title: "Source mirror: docs/plans/active/issue-464-form-numeric-rule/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-464-form-numeric-rule/PLAN.md"
source_sha256: "01490cd1b1fac1857b4958971a6c5a24f93256421181989c5617d2c647e2d957"
mirror_body_sha256: "01490cd1b1fac1857b4958971a6c5a24f93256421181989c5617d2c647e2d957"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #464 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/464
- Fix: FieldRule::Numeric now checks v.is_finite() after parse::<f64>().
