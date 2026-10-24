---
schema_version: 1
id: "MEM-SRC-e8ab1808a81a1a92"
title: "Source mirror: docs/plans/active/issue-456-query-timeout-search-path-leak/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-456-query-timeout-search-path-leak/CHECKLIST.md"
source_sha256: "e8ab1808a81a1a92d3e39225cc44feaa7a2224987ce86d9d48bcb73988e2d59d"
mirror_body_sha256: "e8ab1808a81a1a92d3e39225cc44feaa7a2224987ce86d9d48bcb73988e2d59d"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #456 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/456
- Fix: set_config('search_path', ..., is_local=true) scopes the search_path setting to the active transaction or statement, so dropping the future during timeout/cancel automatically resets the search_path upon statement or transaction end.
