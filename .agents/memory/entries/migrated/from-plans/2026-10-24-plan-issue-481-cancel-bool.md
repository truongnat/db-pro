---
schema_version: 1
id: "MEM-PLAN-issue-481-cancel-bool-81cb012bd65b"
title: "Plan archive: Issue 481 Cancel Bool"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-481-cancel-bool]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "81cb012bd65b12539b0db67ae3f864f682b5db7662fdd48de2a124a95dc32b9a"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-481-cancel-bool/PLAN.md"
  - "docs/plans/active/issue-481-cancel-bool/FINDINGS.md"
  - "docs/plans/active/issue-481-cancel-bool/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-481-cancel-bool/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# #481 — `cancel` ignores `pg_cancel_backend` boolean

## Goal
`PostgresConnector::cancel` must fail when `pg_cancel_backend($1)` returns `false`
(backend already gone or role lacks `pg_signal_backend`). Today `.execute()` hides the
result, so the UI reports Stop succeeded while the statement keeps running.

## Scope
- `crates/infrastructure/src/postgres/connector.rs` `cancel` (~L497): switch to
  `sqlx::query_scalar::<_, bool>` + `fetch_one`; collect refused pids; return one
  `DbError::Internal("could not cancel backend(s) 123, 456")`. Empty refused list → `Ok(())`.
- New pure helper `cancel_failures(refused: Vec<i32>) -> Result<(), DbError>` for testability.
- Tests in `crates/infrastructure/src/postgres/tests/connector_tests.rs` (per-repo `#[path]` split).

## Non-goals
- No retry/force-terminate escalation; `pg_terminate_backend` stays out of scope.
- No live-PG test; error aggregation is covered by the extracted pure fn plus a
  source-shape assertion on the shipped `cancel` body.

## Acceptance
- cancel reports failure naming every refused pid; `Ok(())` when none refused / none active.


## Source: `FINDINGS.md`

# #481 findings

- `connector.rs:497-503` ran `sqlx::query("SELECT pg_cancel_backend($1)").execute(&pool)`
  inside a pid loop and discarded the row — `false` (permission/backend-exited) surfaced
  as `Ok(())`.
- `pg_cancel_backend` returns a single `bool` column; `query_scalar` + `fetch_one`
  exposes it directly. No rows-affected semantics needed.
- `DbError::Internal(String)` is the existing convention for infra-internal failures
  (e.g. `rollback_batch`); used for the aggregated refusal error.
- Refused pids collected across the loop so one error names all of them, matching the
  issue's "could not cancel backend(s) 123, 456" shape. A transport error still aborts
  early via `?`.
- `pids` snapshot comes from `entry.active_backends` — pids may already be stale by the
  time cancel runs, which is exactly why false results must be reported.


## Source: `VERIFICATION.md`

# #481 verification

- `cancel_reports_backends_pg_cancel_backend_refused` — `cancel_failures(vec![123,456])`
  errors and the message names both pids and "cancel".
- `cancel_with_no_refused_backends_succeeds` — empty refused list → `Ok(())`.
- `owned_cancel_reads_pg_cancel_backend_result` — shipped `cancel` body contains
  `query_scalar("SELECT pg_cancel_backend($1)")` → `fetch_one(&pool)` →
  `cancel_failures(refused)`, and no `.execute(&pool)`.
- `cargo check -p db-pro-infrastructure` — clean.
- `cargo test -p db-pro-infrastructure --lib` — 167 passed / 0 failed, including the three
  tests above.

