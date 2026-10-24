---
schema_version: 1
id: "MEM-SRC-536c8bb34954d352"
title: "Source mirror: docs/plans/active/issue-454-pinned-recent-tables-connection-scope/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-454-pinned-recent-tables-connection-scope/CHECKLIST.md"
source_sha256: "536c8bb34954d352afd14c9ad4db437a480f71f652505347837deca4dfdc8899"
mirror_body_sha256: "536c8bb34954d352afd14c9ad4db437a480f71f652505347837deca4dfdc8899"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #454 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/454
- Fix: reset_connection_scope clears pinned_tables and recent_tables so tables from a prior connection are not shown or opened on a newly connected database.
