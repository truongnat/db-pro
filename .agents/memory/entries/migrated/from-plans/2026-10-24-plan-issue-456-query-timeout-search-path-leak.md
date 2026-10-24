---
schema_version: 1
id: "MEM-PLAN-issue-456-query-timeout-search-p-d652af53464b"
title: "Plan archive: Issue 456 Query Timeout Search Path Leak"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-456-query-timeout-search-path-leak]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "d652af53464b65fae02e27093d306df6f0229b95ab9c3c94c3c6c02aae72cd69"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-456-query-timeout-search-path-leak/PLAN.md"
  - "docs/plans/active/issue-456-query-timeout-search-path-leak/FINDINGS.md"
  - "docs/plans/active/issue-456-query-timeout-search-path-leak/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-456-query-timeout-search-path-leak/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #456 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/456
- Fix: set_config('search_path', ..., is_local=true) scopes the search_path setting to the active transaction or statement, so dropping the future during timeout/cancel automatically resets the search_path upon statement or transaction end.


## Source: `FINDINGS.md`

# Issue #456 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/456
- Fix: set_config('search_path', ..., is_local=true) scopes the search_path setting to the active transaction or statement, so dropping the future during timeout/cancel automatically resets the search_path upon statement or transaction end.


## Source: `VERIFICATION.md`

# Issue #456 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/456
- Fix: set_config('search_path', ..., is_local=true) scopes the search_path setting to the active transaction or statement, so dropping the future during timeout/cancel automatically resets the search_path upon statement or transaction end.

