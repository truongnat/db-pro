---
schema_version: 1
id: "MEM-SRC-74ce1e0a338891c1"
title: "Source mirror: docs/plans/active/issue-456-query-timeout-search-path-leak/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-456-query-timeout-search-path-leak/PLAN.md"
source_sha256: "74ce1e0a338891c15d07527265b4c7128aeeb163a60c2d7f197048203ced21a7"
mirror_body_sha256: "74ce1e0a338891c15d07527265b4c7128aeeb163a60c2d7f197048203ced21a7"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #456 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/456
- Fix: set_config('search_path', ..., is_local=true) scopes the search_path setting to the active transaction or statement, so dropping the future during timeout/cancel automatically resets the search_path upon statement or transaction end.
