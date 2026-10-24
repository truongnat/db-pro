---
schema_version: 1
id: "MEM-SRC-5fd4a89850e10ad1"
title: "Source mirror: docs/plans/active/issue-454-pinned-recent-tables-connection-scope/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-454-pinned-recent-tables-connection-scope/FINDINGS.md"
source_sha256: "5fd4a89850e10ad18d8f428c37419a9994baa6777055dfdf1f921196710fca76"
mirror_body_sha256: "5fd4a89850e10ad18d8f428c37419a9994baa6777055dfdf1f921196710fca76"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #454 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/454
- Fix: reset_connection_scope clears pinned_tables and recent_tables so tables from a prior connection are not shown or opened on a newly connected database.
