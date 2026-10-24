---
schema_version: 1
id: "MEM-SRC-667743e56a3b2008"
title: "Source mirror: docs/plans/active/issue-481-cancel-bool/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-481-cancel-bool/PLAN.md"
source_sha256: "667743e56a3b2008d0921deaf5e95d726cced1b1bc728acd39ea7efd11f9d21f"
mirror_body_sha256: "667743e56a3b2008d0921deaf5e95d726cced1b1bc728acd39ea7efd11f9d21f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
