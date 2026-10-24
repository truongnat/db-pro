---
schema_version: 1
id: "MEM-SRC-32e675faab0cacff"
title: "Source mirror: docs/plans/active/data-diff-typed-values/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/data-diff-typed-values/VERIFICATION.md"
source_sha256: "32e675faab0cacff21b38632b4cb39b84aeb86fab391d879fc8400ebe5c27328"
mirror_body_sha256: "32e675faab0cacff21b38632b4cb39b84aeb86fab391d879fc8400ebe5c27328"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification — #457 / #485

Baseline: `76b120724a7542de8b587a002811ddacad28156a`; candidate is the uncommitted working-tree changes based on that SHA. Main must replace candidate identity with the final commit SHA when recording results.

## Parent execution — authoritative integration update

- Exact-baseline standalone public SQLite service probe: exit 101, `Validation("duplicate comparison key in sample: a\u{1f}b\u{1f}c")` (`artifact://55`). This is a runtime reproduction, not merely a source inference.
- Candidate independent standalone public-service probes: SQLite PASS and PostgreSQL 18 PASS, exit 0, each applying seven generated SQL statements with exactly one affected row and recomparing seven equal rows. Reran after the final five-argument UPDATE rendering adjustment; both providers still PASS.
- PostgreSQL permanent real-provider test explicitly executed with PGHOST=127.0.0.1, PGPORT=25432, PGUSER=dbpro, PGDATABASE=dbpro_smoke, DATA_DIFF_TARGET_DATABASE=dbpro_smoke_target and temporary PGPASSWORD: 1 passed / 0 failed / 0 ignored, exit 0. Ignored-by-default is not counted as evidence until this explicit run.
- Integrated `cargo +nightly fmt --all && cargo check --workspace && cargo test --workspace`: exit 0, 1,999 passed / 0 failed / 43 ignored (`artifact://93`). Later final focused test/runtime checks cover the parameter-count correction.
- Independent source review ReviewDataSql accepted this cluster without introduced findings. Provider-specific generated-DDL findings were in a separate cluster.
- Scanner found an introduced sixth `sync_update` argument; fixed without suppression, extra allocations or recomputing changes. UPDATE uses `DataColumnChange.source_value: Option<String>` after typed conversion: None is SQL NULL; Some preserves exact scalar text and escapes apostrophes. Internal equality/keys/predicates remain typed. This is not sentinel/display decoding; text ∅ stays literal text.
- No live UI claim. PostgreSQL and SQLite each proved independently. Binary SQL literals/MySQL backslash modes remain inherited and unverified.

The worker's NOT RUN statements below describe worker actions; they do not override this observed parent evidence.


## Actually executed in this lane
Source inspection and file edits only. **No build, cargo command, tests, linter, formatter, runtime probe or provider query was executed.** No pass counts or runtime claims are asserted.

## Focused tests for Main
```sh
cargo test -p db-pro-core application::data_diff::tests -- --nocapture
cargo test -p db-pro-infrastructure --test data_diff_smoke sqlite_public_data_diff_typed_values_round_trip -- --exact --nocapture
```
The existing five adjacent core tests remain; six public-service regression cases were added: `null_source_and_empty_set_text_target_are_changed`, `null_and_empty_set_text_keys_remain_distinct`, `empty_set_text_survives_insert_and_null_key_update`, `composite_separator_values_do_not_collide_or_change_predicate_boundaries`, `composite_separator_tuple_difference_is_added_and_removed`, `typed_values_preserve_precision_and_escape_sql_text`. Expected counts are not execution evidence.

## Independent PostgreSQL smoke
Prerequisite: two distinct disposable writable PostgreSQL databases accessible with the same credentials. Parent reported preparing PostgreSQL 18 at `127.0.0.1:25432`, user `dbpro`, databases `dbpro_smoke` and `dbpro_smoke_target`; this lane did not verify readiness. Use the local fixture password through `PGPASSWORD` (not stored here).

```sh
PGHOST=127.0.0.1 PGPORT=25432 PGUSER=dbpro PGDATABASE=dbpro_smoke \
DATA_DIFF_TARGET_DATABASE=dbpro_smoke_target \
cargo test -p db-pro-infrastructure --test data_diff_smoke postgres_public_data_diff_typed_values_round_trip -- --ignored --exact --nocapture
```
This test deliberately fails if required database configuration is absent; it does not silently pass/skip inside its body. The normal suite ignores this external-provider test per repository convention.

## Smoke assertions and safety
Each provider test independently constructs the public DataDiffService over the actual provider connector and two separate connection handles. It creates uniquely named scratch tables in separate databases (SQLite: separate in-memory databases), seeds via bound parameters, compares a seven-row fixture with NULL/text keys, NULL/text payloads, colliding legacy composite tuples, a quoted key and hostile-looking text, added/removed/equal rows. Expected initial states: added=1, removed=1, changed=5, equal=1. It applies all seven generated previews, requires exactly one affected row each, and re-runs the service expecting equal=7 and no changes/previews. Tables are dropped and handles disconnected on success. If an assertion fails, uniquely named PostgreSQL tables can remain for diagnosis; only disposable fixtures should be used.

| Provider | Intended support | Automated evidence | Provider runtime evidence |
|---|---|---|---|
| SQLite | NULL/text/typed tuple comparison and executable scalar preview | NOT RUN | NOT VERIFIED |
| PostgreSQL | same, independently exercised | NOT RUN | NOT VERIFIED |

## Baseline reproduction approach (not executed)
Export baseline `76b120724a7542de8b587a002811ddacad28156a` to a fresh temporary directory, copy only the new `crates/infrastructure/tests/data_diff_smoke.rs` into that exported tree, and run the same provider selectors there. No private implementation symbols are used by the probe. [INFERENCE] The baseline should reject the legacy composite-key collision as a duplicate, before preview execution. For #457 independently, use only the first two fixture rows from `source_rows`/`target_rows`, key columns tenant/item: source NULL vs target text `∅` and the reverse should be two changed rows; [INFERENCE] baseline string comparison reports equality instead. Keep the final candidate fixture intact; do not narrow its acceptance tests to reproduce the baseline.

## Limits
No claim about binary SQL literal correctness or MySQL backslash modes: inherited behavior is deliberately unchanged, not addressed/not verified. No UI or runtime worker interaction was exercised. Combined workspace gates, independent review, final SHA, commit/push/status/changelog and issue closure belong to Main.
