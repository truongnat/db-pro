---
schema_version: 1
id: "MEM-SRC-4f09fb7acfea1cf0"
title: "Source mirror: docs/plans/active/issue-454-pinned-recent-tables-connection-scope/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-454-pinned-recent-tables-connection-scope/PLAN.md"
source_sha256: "4f09fb7acfea1cf01fc9158b766d281be305082df9460214ba6dcba2ca449218"
mirror_body_sha256: "4f09fb7acfea1cf01fc9158b766d281be305082df9460214ba6dcba2ca449218"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #454 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/454
- Fix: reset_connection_scope clears pinned_tables and recent_tables so tables from a prior connection are not shown or opened on a newly connected database.
