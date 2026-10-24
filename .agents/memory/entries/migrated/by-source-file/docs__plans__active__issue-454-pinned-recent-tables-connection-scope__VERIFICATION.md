---
schema_version: 1
id: "MEM-SRC-ec37dc7b1fc8456e"
title: "Source mirror: docs/plans/active/issue-454-pinned-recent-tables-connection-scope/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-454-pinned-recent-tables-connection-scope/VERIFICATION.md"
source_sha256: "ec37dc7b1fc8456efd1e7fc33e11ab33605f9b158dcb1445771ead22fe6eb5b6"
mirror_body_sha256: "ec37dc7b1fc8456efd1e7fc33e11ab33605f9b158dcb1445771ead22fe6eb5b6"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #454 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/454
- Fix: reset_connection_scope clears pinned_tables and recent_tables so tables from a prior connection are not shown or opened on a newly connected database.
