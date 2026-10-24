---
schema_version: 1
id: "MEM-SRC-e451947410631def"
title: "Source mirror: docs/plans/active/issue-482-pg-blocking-pids/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-482-pg-blocking-pids/PLAN.md"
source_sha256: "e451947410631def842579a4cc140d65b719f8811d71832aa828905b738faf08"
mirror_body_sha256: "e451947410631def842579a4cc140d65b719f8811d71832aa828905b738faf08"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #482 — blocker detection must use `pg_blocking_pids`

## Problem
`list_locks` (`crates/infrastructure/src/postgres/monitoring.rs`) computed
`blocker_pid` with a LATERAL join on `pg_locks` matching only resource identity +
`granted` flag. It never checked lock-mode compatibility, ignored fast-path
locks and prepared-xact/group blockers, and `LIMIT 1` picked an arbitrary pid.
Result: compatible locks were reported as blockers and the UI could terminate an
innocent session.

## Fix
Replace the LATERAL subquery with `(pg_catalog.pg_blocking_pids(a.pid))[1]::bigint
AS blocker_pid` — PostgreSQL's authoritative blocker list (mode conflict,
fast-path, group aware). Row shape unchanged: one optional `blocker_pid` per
lock row (the view displays a single blocker per row).

## Files
- `crates/infrastructure/src/postgres/monitoring.rs` — extract `LIST_LOCKS_SQL`
  const; swap subquery for `pg_blocking_pids`; declare `mod tests`.
- `crates/infrastructure/src/postgres/tests/monitoring_tests.rs` — SQL contract
  tests (new file).
