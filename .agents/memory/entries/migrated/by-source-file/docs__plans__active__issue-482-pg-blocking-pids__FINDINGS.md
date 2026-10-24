---
schema_version: 1
id: "MEM-SRC-4be81e88ca9b8389"
title: "Source mirror: docs/plans/active/issue-482-pg-blocking-pids/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-482-pg-blocking-pids/FINDINGS.md"
source_sha256: "4be81e88ca9b8389f7d6cbdb29f5a8b2e73f19af8a826f449fa3fbbabae7817e"
mirror_body_sha256: "4be81e88ca9b8389f7d6cbdb29f5a8b2e73f19af8a826f449fa3fbbabae7817e"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — #482

- Old query joined `pg_locks waiting` × `pg_locks blocking` on all resource
  columns + `waiting.granted = false` + `blocking.granted = true`. Two sessions
  holding/for-awaiting *compatible* modes on the same resource satisfy that
  predicate — false blockers.
- `waiting.granted = false` also misses fast-path locks (never appear ungranted
  in pg_locks) and prepared-transaction blockers (no pid in pg_locks at all).
- `pg_blocking_pids(pid)` (PG ≥ 9.6) returns the authoritative array, including
  group leaders for parallel queries.
- `MonitorLock.blocker_pid` is `Option<i64>` and every consumer renders a single
  blocker → `[1]` keeps the shape.
- Column order of `list_locks` SELECT unchanged: `locked_pid, blocker_pid,
  relation, lock_type, mode, granted, waiting_ms` — row mapping untouched.
