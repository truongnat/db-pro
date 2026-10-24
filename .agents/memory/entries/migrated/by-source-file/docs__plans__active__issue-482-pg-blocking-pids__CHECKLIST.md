---
schema_version: 1
id: "MEM-SRC-9096118e9d73381e"
title: "Source mirror: docs/plans/active/issue-482-pg-blocking-pids/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-482-pg-blocking-pids/CHECKLIST.md"
source_sha256: "9096118e9d73381ec1a2f83ea58176b638db4ce98c321de1da266c28f68aaffe"
mirror_body_sha256: "9096118e9d73381ec1a2f83ea58176b638db4ce98c321de1da266c28f68aaffe"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — #482

- [x] `LIST_LOCKS_SQL` uses `pg_blocking_pids(a.pid)`, resource-join subquery removed
- [x] Row shape/`blocker_pid` Option semantics preserved for the view
- [x] Contract test asserts `pg_blocking_pids` present, `blocking.granted` gone
- [x] `cargo check -p db-pro-infrastructure`
- [x] `cargo test -p db-pro-infrastructure monitoring`
