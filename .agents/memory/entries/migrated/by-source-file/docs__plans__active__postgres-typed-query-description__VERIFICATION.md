---
schema_version: 1
id: "MEM-SRC-d9590f134f895ff9"
title: "Source mirror: docs/plans/active/postgres-typed-query-description/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/postgres-typed-query-description/VERIFICATION.md"
source_sha256: "d9590f134f895ff9f17b98da43c3f65835f1c43f19357f54ec3722547c7424ef"
mirror_body_sha256: "d9590f134f895ff9f17b98da43c3f65835f1c43f19357f54ec3722547c7424ef"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
