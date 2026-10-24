---
schema_version: 1
id: "MEM-SRC-c14694b78039f037"
title: "Source mirror: docs/plans/active/issue-481-cancel-bool/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-481-cancel-bool/FINDINGS.md"
source_sha256: "c14694b78039f037eb0a8c22d84564a6b4046fb99a9a533438a61e2f23eb0095"
mirror_body_sha256: "c14694b78039f037eb0a8c22d84564a6b4046fb99a9a533438a61e2f23eb0095"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
