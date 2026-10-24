---
schema_version: 1
id: "MEM-PLAN-issue-449-er-spatial-index-long--f066b6c4dcee"
title: "Plan archive: Issue 449 Er Spatial Index Long Edges"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-449-er-spatial-index-long-edges]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "f066b6c4dceec3b434ea41c5c5a65c0e10ef5c2363dcc706a9e9b7ea9b964299"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-449-er-spatial-index-long-edges/PLAN.md"
  - "docs/plans/active/issue-449-er-spatial-index-long-edges/FINDINGS.md"
  - "docs/plans/active/issue-449-er-spatial-index-long-edges/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-449-er-spatial-index-long-edges/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #449 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/449
- Fix: insert_edge samples cells across the span when spanning > 32 cells rather than truncating from min_corner, ensuring far endpoints and mid-spans remain queryable.


## Source: `FINDINGS.md`

# Issue #449 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/449
- Fix: insert_edge samples cells across the span when spanning > 32 cells rather than truncating from min_corner, ensuring far endpoints and mid-spans remain queryable.


## Source: `VERIFICATION.md`

# Issue #449 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/449
- Fix: insert_edge samples cells across the span when spanning > 32 cells rather than truncating from min_corner, ensuring far endpoints and mid-spans remain queryable.

