---
schema_version: 1
id: "MEM-PLAN-postgres-typed-query-description-f6a908bacd20"
title: "Plan archive: Postgres Typed Query Description"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, postgres-typed-query-description]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "f6a908bacd20bf35d6857d3508b92907ba6b16368469e7f627c33b1da6b0a981"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/postgres-typed-query-description/PLAN.md"
  - "docs/plans/active/postgres-typed-query-description/FINDINGS.md"
  - "docs/plans/active/postgres-typed-query-description/VERIFICATION.md"
  - "docs/plans/active/postgres-typed-query-description/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/postgres-typed-query-description/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

# #533 verification and standalone smoke recipe

Baseline: 76b120724a7542de8b587a002811ddacad28156a. Published candidate: 6f94372dd5caba22e525fbc1aff918b936e0491d; push and #533 closure succeeded. State: REVIEW due inherited lint gate, not missing provider proof.

## Parent final execution — authoritative integration result

- Final typed prepare/unspecified-NULL candidate: `cargo +nightly fmt --all -- --check` PASS, exit 0.
- `DATABASE_URL=postgres://...@127.0.0.1:25432/dbpro_smoke cargo test --locked -p db-pro-infrastructure --test pg_integration pg_typed_query_ -- --ignored --test-threads=1`: **5 passed / 0 failed / 0 ignored**, exit 0 (`artifact://159`). This explicitly executes the normally ignored real-provider cases, including slow tail and contextual NULL; ignored defaults are not counted as passes.
- `cargo test -p db-pro-infrastructure --lib postgres::query_mapper::tests`: 16 passed / 0 failed / 0 ignored, exit 0 (`artifact://159`).
- Final original public INTEGER composite-PK DbProRuntime probe: `provider generated SQL pk: PASS`, exit 0. Same INTEGER fixture and Int64 inputs as the original failure; no BIGINT workaround.
- `cargo clippy -p db-pro-core -p db-pro-infrastructure -p db-pro-runtime --all-targets -- -D warnings`: PASS, exit 0. `cargo test --workspace`: **2,003 passed / 0 failed / 50 ignored**, exit 0 (`artifact://160`).
- `cargo check --workspace` previously PASS on integrated source. Workspace all-target Clippy FAIL remains the five unchanged app diagnostics (`artifact://143`); no claim that this global gate passes.
- Independent reviewer ReviewPgTypedMetadata final verdict ACCEPT (source scope), no unresolved introduced P0/P1/P2 after both tail-drain and NULL-type corrections. Reviewer did not execute tests; actual provider results are parent-owned above.
- Earlier E0309 generic bound and 4-pass/1-fail integer=text runs are retained as real intermediate failures below; final passes do not erase them.
- Published exact candidate SHA above and closed #533 with observed evidence. No CI or live UI claim. Different binary parameter types for identical SQL remain the existing SQLx cache limitation.

Worker-only NOT RUN notes below are historical and superseded for integration by this section.


## Observed inherited failure (parent evidence)

Main reported this command exited **1** against PostgreSQL 18:

```sh
DB_PRO_DISABLE_KEYRING=1 PROBE_STATE=/tmp/dbpro-generated-pg-pk-state PROBE_CASE=pk PROBE_CONFIG='{"name":"generated-pg","host":"127.0.0.1","port":25432,"database":"dbpro_smoke","username":"dbpro","driver":"postgres","ssl_mode":"disable","ssh_tunnel":null}' PROBE_PASSWORD=dbpro_fixture_only cargo run --offline --manifest-path /tmp/db-pro-cluster-probes/candidate-generated-sql/Cargo.toml
```

Exact reported output:

```text
Error: QueryFailed("incorrect binary data format in bind parameter 1")
```

Parent supplied this observation as ground truth; worker inspected `/tmp/db-pro-cluster-probes/candidate-generated-sql/src/main.rs`, including INTEGER composite PK setup and public runtime lookup with Int64(10/21). No raw artifact URI was supplied by background delivery; no artifact://69 claim is made. This inherited P1 is tracked separately as #533. The worker did not rerun the failure.

## Candidate tests (NOT RUN)

Five permanent tests are in `crates/infrastructure/tests/pg_integration.rs`, prefixed `pg_typed_query_`:

1. `integer_composite_pk_via_table_data_service`: real service, real registry/meta repository and PostgreSQL connector, same backend transaction, missing-first metadata followed by matching/missing/matching service lookups, INTEGER keys, BIGINT i64::MAX / i64::MAX-1 precision, nullable flags, parameterized execute and explain, rollback cleanup.
2. `uncast_parameter_preserves_bigint_precision_and_empty_metadata`: repeated identical SQL and parameter types, i64::MAX/i64::MIN, present/empty/present results, INT8 metadata.
3. `keeps_metadata_at_streaming_row_limits`: max_rows 0/2, typed metadata, bounded retained rows, follow-up connection usability.
4. `cap_returns_before_slow_tail_without_schema_override`: no explicit session/schema override; two 32 KiB rows force wire flushing before the unused five-second pg_sleep tail, under a two-second query timeout. Success requires returning the capped rows rather than synchronously draining the tail.
5. `null_parameters_preserve_contextual_inference`: first-execution INTEGER/BOOL/DATE comparisons with untyped NULL, plus a mixed NULL/Int64 query. NULL must remain unspecified instead of receiving a TEXT preparation hint.

`postgres/tests/query_mapper_tests.rs` additionally checks captured types for converted integer/date/timestamp/time/offset-time parameters and unspecified OID 0 for domain NULL. The capture follows the existing conversion policy, not a second type dispatch.

Independent review rejected the prior execution-before-describe candidate because Describe synchronously drained a truncated stream. The revised candidate uses prepare_with before Describe and execution. Main reported the original public INTEGER probe and first three tests passed on the prior candidate; those results do not verify this revised implementation or the fourth regression. Main must rerun and record exact command/count/SHA evidence.

Main subsequently reported its in-progress owned clippy compile failed with E0309 at bind_value: generic T needed to outlive 'q. Worker applied the suggested `+ 'q` bound; did not rerun the gate. Exact parent command/counts were not supplied to the worker. This failure is recorded, not treated as a passing gate; the final revised snapshot still needs Main verification.

Follow-up review resolved the synchronous tail-draining finding in source but identified untyped NULL being captured as TEXT. Worker corrected that hint to protocol OID 0 and added the fifth integration regression; no gate was executed by the worker. Revised verification/re-review is still required.

Main reported a live-PG run of the pre-NULL-fix snapshot (`bg51`, parent artifact://147): **4 passed / 1 failed / 0 ignored**. The failing `pg_typed_query_null_parameters_preserve_contextual_inference` reported `QuerySyntaxAt` for `integer = text`, position 10, at test line 1690. This is ground-truth failure evidence for the reviewer finding, not a final-candidate result. Worker did not rerun it; the corrected OID 0 snapshot awaits Main's next run. Exact command/exit status were not supplied to the worker.

Existing ignored real-PostgreSQL test convention is retained; these tests require DATABASE_URL but no preloaded fixture schema. Unique table names avoid #441 fixture names.

Main should run, once after all sibling edits land:

```sh
DATABASE_URL='postgres://dbpro:dbpro_fixture_only@127.0.0.1:25432/dbpro_smoke' \
  cargo test --offline --locked -p db-pro-infrastructure --test pg_integration pg_typed_query_ -- --ignored --test-threads=1
```

Formatting, broader unit/provider/session/cancellation regression checks and exact final counts belong to Main. This worker ran **no builds, tests, formatters, linters or smoke executions**. No passing counts, CI IDs or candidate runtime claims are available.

## Standalone public DbProRuntime evidence

The parent retained and reran the original independent generated-SQL public-service probe without changing its INTEGER fixture to BIGINT. It creates a composite INTEGER PK and exercises matching/missing TableDataService::fetch_row_by_pk with Int64 parameters. The initial run failed with the binary bind format error above; final-candidate rerun prints `provider generated SQL pk: PASS`, exit 0.

No separate unexecuted smoke executable is retained in this plan. Permanent real-PG tests above cover repeated lookup, precise BIGINT extrema, contextual NULL, nullable/empty metadata, row limits and capped slow-tail completion. Temporary probes/state are removed after evidence collection. No live SQLite behavior is inferred from this PostgreSQL-only change.

## Provider disposition

PostgreSQL final automated/live checks: PASS as recorded above. SQLite N/A for this PostgreSQL-only preparation change; no cross-provider equivalence inferred. Identical SQL with different binary parameter types remains the inherited SQLx cache limitation.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — #533 PostgreSQL typed query description

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | PostgresTypedDescribe / implementation worker |
| Issue(s) | #533 — inherited P1 prerequisite discovered during #441/#442 runtime verification |
| Task state | Review |
| Baseline SHA | 76b120724a7542de8b587a002811ddacad28156a |
| Branch / PR | shared main; Main alone integrates, publishes and closes |
| Scope interpretation | Align query description and actual argument wire types without duplicating conversion or disabling caches. Add real PostgreSQL regressions and an external public runtime smoke recipe. |
| Out of scope | schema DDL, other-provider introspection, core data diff, ports, unrelated sessions, shared manifests/changelog/status |

## 2. Progress checkpoint

- Exact claimed baseline: `76b120724a7542de8b587a002811ddacad28156a`. No git command was run to rediscover shared HEAD. Candidate is uncommitted; Main must supply final SHA and account for independent shared-main changes.
- Completed implementation acceptance: opt-in capture of actual converted non-null argument wire types and unspecified hints for untyped NULL; public prepare_with before describe and streaming; nullable/empty metadata without tail draining; bounded retained rows; permanent INTEGER composite-PK public-service integration regression; BIGINT extrema; repeated same-typed SQL; execute/explain coverage; slow-tail regression; contextual NULL regression; captured-type unit test; standalone DbProRuntime smoke source/recipe.
- Remaining acceptance: actual candidate build/test/format/lint/provider execution and independent review are intentionally assigned to Main. No worker verification was authorized.
- Inherited P1: baseline `crates/infrastructure/src/postgres/connector.rs:376-381` describes without actual parameter types before binding binary i64. Parent-observed error and command preserved in VERIFICATION.md; separately tracked as #533, not attributed to candidate #441/#442.
- Tests already run: none by worker; commands/counts/exit statuses n/a. Parent inherited reproduction exited 1 as reported. Parent also reported an in-progress clippy compile failure E0309 in bind_value; worker added the required `T: 'q` bound without rerunning. Exact parent gate command/counts not supplied; final revised gates remain pending.
- Parent additionally observed the pre-NULL-fix live-PG run: 4 passed / 1 failed / 0 ignored; contextual NULL regression failed with QuerySyntaxAt integer = text, position 10 (bg51, parent artifact://147). Worker accepted this as ground truth and did not rerun; OID 0 fix still awaits Main verification.
- Dependency change: none. SQLx 0.8.6 official API and installed implementation inspected; ordinary statement caching retained.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | Baseline 76b120724a7542de8b587a002811ddacad28156a; candidate uncommitted, final integration SHA pending Main |
| Commit list | n/a; worker does not commit |
| File / surface inventory | PostgreSQL connector.rs: typed prepare/describe/bounded-fetch helper; query_mapper.rs: opt-in type capture with unspecified NULL; adjacent mapper tests; pg_integration.rs: five opt-in real-PG regressions; canonical plan/evidence records. Temporary smoke source removed after proof. |
| Acceptance mapping | INTEGER PK/metadata/precision/execute/explain, row limits, capped slow tail and contextual NULL map to five real-PG tests; type hints map to mapper unit case; public DbProRuntime original INTEGER probe observed in VERIFICATION.md. |
| Commands and counts | No builds/tests/lint/format/smoke executed by worker; no pass counts claimed. Suggested focused command in VERIFICATION.md. |
| CI run IDs / status | not run |
| Known limitations | SQLx's existing SQL-only cache still cannot safely change binary parameter types for identical SQL. Candidate does not claim otherwise. Live verification pending. |
| Migrations / config implications | none; DATABASE_URL for opt-in existing-style integration tests; smoke uses disposable PROBE_STATE and local-only fixture credentials |
| Out-of-scope changes | none |

## 4. Review outcome

Independent review rejected the prior uncommitted execution-before-describe candidate with one introduced P2: Describe synchronously drained the unused tail after reaching max_rows. Revised prepare_with-before-execution design resolves that finding in source. Follow-up review identified another P2: untyped NULL captured as TEXT; worker corrected it to unspecified OID 0 and added regression coverage. Independent final re-review requested. Inherited P1 count: 1 (#533). No accepted final SHA or final verdict is available. Main must run gates, obtain revised review, pin the SHA and decide closure.

## 5. Research / audit handoff

- Source date: 2026-10-16.
- Source references: baseline connector.rs:218,344-469,689,814-819,932-979; query_mapper.rs:10-64,305-329; core TableDataService:103-120; Cargo.lock SQLx 0.8.6. Local installed SQLx executor.rs:168-248,445-485; arguments.rs:83-103; types/int.rs:134-151.
- Official API: https://docs.rs/sqlx/0.8.6/sqlx/trait.Executor.html#tymethod.prepare_with . Implementation references and root-cause chain are listed in FINDINGS.md.
- Factual finding: parent observed INTEGER/Int64 public-service failure; installed SQLx caches by SQL, describes with empty parameter hints, and normally executes with actual argument types.
- Inference: the INT4/i64 wire mismatch is the cause, grounded in that source chain and parent observation; the candidate remedy still awaits runtime proof.
- Decision: opt-in generic capture of actual converted non-null argument types, unspecified Parse OID 0 for domain NULL, then prepare_with before describe and streaming. Earlier execution-before-describe design was rejected for synchronous tail draining; a later TEXT-hint-for-NULL regression was corrected to preserve contextual inference. Other two describe callsites lack typed parameters; execute/explain keep the non-capturing bind path and are unchanged.
- Unresolved questions: none in implementation scope; final integrated SHA and verification outcomes pending Main.
- Downstream task activated: Main verification/review/closure of #533; then original #441/#442 runtime acceptance.

## 6. Tổng kết

Đã dùng generic helper để thu kiểu tham số không-null theo chuyển đổi hiện có; NULL không định kiểu giữ OID 0 để PostgreSQL suy luận theo ngữ cảnh. prepare_with và describe đều chạy trước streaming, không chờ slow tail sau max_rows. Hai P2 do reviewer phát hiện đã được xử lý ở nguồn: drain kết quả và hint TEXT cho NULL. Có tổng cộng 5 integration test PostgreSQL thật, unit test capture kiểu và smoke DbProRuntime độc lập. P1 kế thừa ghi riêng ở #533. Worker chưa chạy gates; Main cần chạy ứng viên mới, ghim SHA và lấy review cuối. Việc đổi kiểu binary cho SQL đã cache vẫn ngoài phạm vi.

