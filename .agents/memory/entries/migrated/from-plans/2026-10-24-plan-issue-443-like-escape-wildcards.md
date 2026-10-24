---
schema_version: 1
id: "MEM-PLAN-issue-443-like-escape-wildcards-a12148084f18"
title: "Plan archive: Issue 443 Like Escape Wildcards"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-443-like-escape-wildcards]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "a12148084f188b973dcc64b983c84f5e9764440394eb3aec75061a13c0af64ed"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-443-like-escape-wildcards/PLAN.md"
  - "docs/plans/active/issue-443-like-escape-wildcards/FINDINGS.md"
  - "docs/plans/active/issue-443-like-escape-wildcards/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-443-like-escape-wildcards/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #443 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/443
- Fix: escape_like_pattern escapes %, _, and \ in user input for Contains/StartsWith/EndsWith filters, and text_like appends ESCAPE clause to prevent wildcards from treating user literals as wildcards.


## Source: `FINDINGS.md`

# Issue #443 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/443
- Fix: escape_like_pattern escapes %, _, and \ in user input for Contains/StartsWith/EndsWith filters, and text_like appends ESCAPE clause to prevent wildcards from treating user literals as wildcards.


## Source: `VERIFICATION.md`

# Issue #443 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/443
- Fix: escape_like_pattern escapes %, _, and \ in user input for Contains/StartsWith/EndsWith filters, and text_like appends ESCAPE clause to prevent wildcards from treating user literals as wildcards.

