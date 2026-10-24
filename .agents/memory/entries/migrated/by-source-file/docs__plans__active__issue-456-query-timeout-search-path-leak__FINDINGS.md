---
schema_version: 1
id: "MEM-SRC-33e7ffbdabf624a8"
title: "Source mirror: docs/plans/active/issue-456-query-timeout-search-path-leak/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-456-query-timeout-search-path-leak/FINDINGS.md"
source_sha256: "33e7ffbdabf624a83e78876c311d95b97eaffac455852abf5646131624e12cc9"
mirror_body_sha256: "33e7ffbdabf624a83e78876c311d95b97eaffac455852abf5646131624e12cc9"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #456 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/456
- Fix: set_config('search_path', ..., is_local=true) scopes the search_path setting to the active transaction or statement, so dropping the future during timeout/cancel automatically resets the search_path upon statement or transaction end.
