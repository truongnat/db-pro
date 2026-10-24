---
schema_version: 1
id: "MEM-PLAN-issue-455-er-layout-coordinate-v-6946cd8b0866"
title: "Plan archive: Issue 455 Er Layout Coordinate Validation"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-455-er-layout-coordinate-validatio]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "6946cd8b086614a7f75eb4d0efc649c396e2703a375277bda7d6f43278755074"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-455-er-layout-coordinate-validation/PLAN.md"
  - "docs/plans/active/issue-455-er-layout-coordinate-validation/FINDINGS.md"
  - "docs/plans/active/issue-455-er-layout-coordinate-validation/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-455-er-layout-coordinate-validation/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #455 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/455
- Fix: Sanitizes coordinates in restore_diagram_layouts and apply_position_overrides, rejecting NaN, Inf, and coordinates > 1,000,000 to prevent canvas blanking.


## Source: `FINDINGS.md`

# Issue #455 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/455
- Fix: Sanitizes coordinates in restore_diagram_layouts and apply_position_overrides, rejecting NaN, Inf, and coordinates > 1,000,000 to prevent canvas blanking.


## Source: `VERIFICATION.md`

# Issue #455 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/455
- Fix: Sanitizes coordinates in restore_diagram_layouts and apply_position_overrides, rejecting NaN, Inf, and coordinates > 1,000,000 to prevent canvas blanking.

