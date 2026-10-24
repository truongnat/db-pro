---
schema_version: 1
id: "MEM-SRC-bc65eb7810b72f07"
title: "Source mirror: docs/plans/active/issue-456-query-timeout-search-path-leak/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-456-query-timeout-search-path-leak/VERIFICATION.md"
source_sha256: "bc65eb7810b72f07ed76e8dd761bcdc2c6dfec67b5528b675e14a655b198a75b"
mirror_body_sha256: "bc65eb7810b72f07ed76e8dd761bcdc2c6dfec67b5528b675e14a655b198a75b"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #456 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/456
- Fix: set_config('search_path', ..., is_local=true) scopes the search_path setting to the active transaction or statement, so dropping the future during timeout/cancel automatically resets the search_path upon statement or transaction end.
