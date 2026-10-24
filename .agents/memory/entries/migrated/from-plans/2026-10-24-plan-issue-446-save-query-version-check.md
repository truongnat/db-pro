---
schema_version: 1
id: "MEM-PLAN-issue-446-save-query-version-che-286253c9229a"
title: "Plan archive: Issue 446 Save Query Version Check"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-446-save-query-version-check]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "286253c9229ac08eac01b47ba5ceed7f2cc3efe8bbc3134db25dace9bf76b47f"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-446-save-query-version-check/PLAN.md"
  - "docs/plans/active/issue-446-save-query-version-check/FINDINGS.md"
  - "docs/plans/active/issue-446-save-query-version-check/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-446-save-query-version-check/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #446 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/446
- Fix: commit_dispatched records the document's buffer version along with the request. on_query_saved only calls mark_saved if the buffer version still matches, preventing edits made while save was in flight from being marked as saved.


## Source: `FINDINGS.md`

# Issue #446 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/446
- Fix: commit_dispatched records the document's buffer version along with the request. on_query_saved only calls mark_saved if the buffer version still matches, preventing edits made while save was in flight from being marked as saved.


## Source: `VERIFICATION.md`

# Issue #446 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/446
- Fix: commit_dispatched records the document's buffer version along with the request. on_query_saved only calls mark_saved if the buffer version still matches, preventing edits made while save was in flight from being marked as saved.

