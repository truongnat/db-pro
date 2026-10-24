---
schema_version: 1
id: "MEM-PLAN-issue-482-pg-blocking-pids-4b65d1dae651"
title: "Plan archive: Issue 482 Pg Blocking Pids"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-482-pg-blocking-pids]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "4b65d1dae6513e2c412ec875be4c4597496eaf1e7eaf67102b01bfae44985f01"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-482-pg-blocking-pids/PLAN.md"
  - "docs/plans/active/issue-482-pg-blocking-pids/FINDINGS.md"
  - "docs/plans/active/issue-482-pg-blocking-pids/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-482-pg-blocking-pids/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

# Verification — #482

- `cargo check -p db-pro-infrastructure` — clean (with the rest of the touched
  workspace crates).
- `cargo test -p db-pro-infrastructure monitoring` — `monitoring_tests` pass.
- Not run against a live PostgreSQL: no runtime fixture available; correctness
  rests on `pg_blocking_pids` being the documented server-side answer
  (https://www.postgresql.org/docs/current/functions-info.html) plus the
  SQL-shape contract tests.

