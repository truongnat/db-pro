---
schema_version: 1
id: "MEM-SRC-2664aef3d2d18003"
title: "Source mirror: docs/plans/active/postgres-typed-query-description/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/postgres-typed-query-description/PLAN.md"
source_sha256: "2664aef3d2d18003566ff7dd9176212e4d94d98e1d04a4e4b3fc18ad08d79b33"
mirror_body_sha256: "2664aef3d2d18003566ff7dd9176212e4d94d98e1d04a4e4b3fc18ad08d79b33"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# PostgreSQL typed query description

State: REVIEW — verified fix published at 6f94372dd5caba22e525fbc1aff918b936e0491d; #533 closed completed. Five real-PG cases, 16 mapper tests and original public INTEGER probe PASS; inherited workspace lint gate remains failed.

## Scope

Separate inherited P1 prerequisite tracked by [#533](https://github.com/truongnat/db-pro/issues/533), uncovered during #441/#442 verification: `TableDataService::fetch_row_by_pk` with an INTEGER composite primary key and `CellValue::Int64(10/21)` fails with `incorrect binary data format in bind parameter 1` on PostgreSQL 18. Baseline: `76b120724a7542de8b587a002811ddacad28156a`. This is not a generated-SQL dialect regression. Issue closure belongs to Main.

The candidate changes PostgreSQL query preparation only. One shared conversion dispatch binds each converted non-null value and optionally captures SQLx's actual `Encode::produces` / `Type::type_info` selection. Domain NULL has no declared type and uses PostgreSQL Parse OID 0 to retain contextual inference; its existing null payload encoding is unchanged. Only query requests a separate parameter-type vector; execute/explain callers do not allocate it. `fetch_typed_query` calls public `prepare_with(sql, types)`, then `describe` for complete nullable/empty-result metadata, then the existing bounded fetch/map stream. There is no post-stream describe or synchronous unused-tail drain. A zero row limit prepares/describes but does not poll execution, preserving the baseline contract. No dependency, cache clear, cache disabling, retries, or SQL special cases.

## Invariants and acceptance

- INTEGER composite-PK lookup through the real TableDataService: matching, missing and repeated matching keys, with Int64 parameters.
- BIGINT/i64 extrema retain precision; uncast parameter output is described as INT8.
- Empty results retain complete column metadata and nullable flags.
- First-execution untyped NULL comparisons retain integer, boolean and date contextual inference, including mixed NULL/Int64 parameters.
- Repeated SQL with the same bound parameter types uses normal SQLx caching.
- `max_rows` limits retained/mapped rows, including zero; a dropped bounded stream leaves the connection usable. Fast capped rows return without waiting for an unused slow tail.
- Backend cancellation registration, timeout, search_path push/pop and transaction-lease cleanup remain in their existing enclosing scopes.
- Execute/explain retain their already-typed preparation paths; integration coverage exercises both.

## Existing cache contract

SQLx 0.8.6 caches statements by SQL string, not by parameter-type vector. Reusing identical SQL with a different binary parameter type is an existing limitation across query/execute/explain. This candidate does not claim to fix that limitation and does not introduce type-specific SQL cache keys.

## Non-goals and provider matrix

No changes to generated SQL, schema DDL, other providers, core data diff, ports, session policy, shared manifests/changelog/status or unrelated work.

| Provider | Scope | Automated / live evidence |
|---|---|---|
| PostgreSQL | typed query preparation ordering | 5 real-provider cases + original public-service probe PASS; mapper suite 16 PASS |
| SQLite | not affected: no SQLx PostgreSQL prepared cache | N/A; no SQLite behavior claim |

Baseline claims: 76b120724a7542de8b587a002811ddacad28156a. Final implementation: 6f94372dd5caba22e525fbc1aff918b936e0491d. Publication and issue closure succeeded; feature plan stays REVIEW rather than falsely asserting all repository gates are green.
