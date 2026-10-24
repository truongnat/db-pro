---
schema_version: 1
id: "MEM-SRC-c31678e49f8138fb"
title: "Source mirror: docs/plans/active/postgres-typed-query-description/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/postgres-typed-query-description/FINDINGS.md"
source_sha256: "c31678e49f8138fbb63da5ac0359e5b1921d28f1decfd29801cc0fd440849d67"
mirror_body_sha256: "c31678e49f8138fbb63da5ac0359e5b1921d28f1decfd29801cc0fd440849d67"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #533 findings and decision

## Inherited P1 — INTEGER composite-PK lookup fails

Baseline: `76b120724a7542de8b587a002811ddacad28156a`. Parent-observed public `DbProRuntime` failure, not re-executed by this worker; exact command/result in VERIFICATION.md. It blocks normal INTEGER table row lookup, not just a test fixture. Tracking: https://github.com/truongnat/db-pro/issues/533 . Candidate behavior is not yet verified.

Baseline source chain:

1. `crates/core/src/application/table_data_service.rs:103-120` builds a validated composite-PK query and dispatches supplied values.
2. `crates/infrastructure/src/postgres/query_mapper.rs:10-16` binds QueryParam::Int64 using SQLx i64 encoding.
3. `crates/infrastructure/src/postgres/connector.rs:376-381` calls untyped `describe(sql)` before `query_with(sql, pg_args)`.

Installed/locked dependency SQLx 0.8.6 implementation:

- `sqlx-postgres/src/arguments.rs:83-103`: argument encoding records each actual produced type; those types are private, not available through a public PgArguments getter.
- `sqlx-postgres/src/connection/executor.rs:168-195`: get_or_prepare looks up cached statements by SQL alone, ignoring a newly supplied parameter type vector on a cache hit.
- Same file `:212-248`: argument-bound execution normally prepares using argument types, then sends binary parameters.
- Same file `:465-483`: describe calls get_or_prepare with an empty type vector and then computes nullability.
- `sqlx-postgres/src/types/int.rs`: i64 is INT8 and encodes its 8-byte big-endian representation.

**Root-cause inference grounded in those paths and the observed error:** describing an INTEGER equality predicate first caches server-inferred INT4 parameter metadata. Later Int64 binary arguments cannot be decoded as INT4. The shared mapper is correct to preserve i64 rather than narrowing user values.

## Decision

Use public `prepare_with` with the actual non-null argument types before description and execution. One shared conversion dispatch calls generic `bind_value`, which optionally captures `value.produces().unwrap_or_else(T::type_info)` using exactly SQLx add's type selection after conversion. There is no second QueryParam-to-type dispatch. Domain NULL is intentionally unspecified (Parse OID 0), not TEXT: it has no declared type or wire payload and must retain contextual server inference. Existing bind-only callers pass None and allocate no additional vector. Query opts into type capture, prepares the typed cached statement, describes its nullable/empty metadata, then streams with the original row limit.

The earlier execution-before-describe candidate was rejected by independent review: post-stream `describe` calls SQLx `wait_until_ready`, synchronously draining an unused result tail and potentially timing out an otherwise promptly capped result. That P2 is addressed at source level by moving all prepare/describe work before execution. A fourth live-PG test sends two wide rows before a five-second tail under a two-second query timeout; it is pending Main execution. This rejected candidate must not be published.

PgArguments.types is private, so the opt-in generic capture is required for public typed preparation. `prepare_with` still reuses SQLx's ordinary SQL-only cache; it is not a solution for changing binary parameter types on cached identical SQL. No clear/disable cache or query-specific workaround.

Follow-up review found that capturing Option<String>::None as a TEXT hint regressed first-execution numeric/boolean/date NULL comparisons. The revised NULL branch keeps existing null encoding but supplies OID 0 to prepare_with. SQLx `PgTypeInfo::with_oid` preserves that OID and `resolve_type_id` returns it directly. PostgreSQL's Parse protocol explicitly defines zero as unspecified; Bind NULL carries length -1 and no value bytes. A fifth live test covers fresh INTEGER/BOOL/DATE contexts and mixed NULL/Int64 arguments. This correction is pending Main gates/re-review.

Official protocol: https://www.postgresql.org/docs/18/protocol-message-formats.html#PROTOCOL-MESSAGE-FORMATS-PARSE and https://www.postgresql.org/docs/18/protocol-message-formats.html#PROTOCOL-MESSAGE-FORMATS-BIND .

All baseline describe paths were inspected: joined and owned non-parameterized transaction reads (`connector.rs:218` and `:689`) have no typed arguments; they are intentionally unchanged. Query is the only describe-before-bound-execution path. Execute, parameterized transaction execution and explain already prepare from actual arguments. Their session/cancellation policy is not changed.

Existing limitation: identical SQL used with different binary argument types can still reuse a statement cached for the earlier type. This candidate preserves, and does not claim to fix, that SQLx behavior. Repeated identical types with different values are covered.

Official API/source references (read 2026-10-16):

- https://docs.rs/sqlx/0.8.6/sqlx/trait.Executor.html#tymethod.prepare_with
- https://docs.rs/sqlx-postgres/0.8.6/src/sqlx_postgres/connection/executor.rs.html
- https://docs.rs/sqlx-postgres/0.8.6/src/sqlx_postgres/arguments.rs.html
- https://docs.rs/sqlx-postgres/0.8.6/src/sqlx_postgres/types/int.rs.html

Dependency source inspected locally under `/home/vietis/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`; version pinned by Cargo.lock, no dependency changes. All proposed candidate effects remain source-level evidence until Main records a verified integration SHA.
