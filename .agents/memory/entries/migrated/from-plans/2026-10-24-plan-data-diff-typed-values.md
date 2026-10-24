---
schema_version: 1
id: "MEM-PLAN-data-diff-typed-values-29e6bd00fa79"
title: "Plan archive: Data Diff Typed Values"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, data-diff-typed-values]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "29e6bd00fa79e6f435c7bc067d4826290d09004a9bf20238569c9369987bbdd5"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/data-diff-typed-values/PLAN.md"
  - "docs/plans/active/data-diff-typed-values/FINDINGS.md"
  - "docs/plans/active/data-diff-typed-values/VERIFICATION.md"
  - "docs/plans/active/data-diff-typed-values/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/data-diff-typed-values/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Typed data-diff values — #457 / #485

State: REVIEW. Baseline: `76b120724a7542de8b587a002811ddacad28156a`. Parent independently executed real SQLite/PostgreSQL round trips and integrated regression gates; source review ACCEPT. Publication belongs to parent.

## Requirements and context
At the baseline, `crates/core/src/application/data_diff.rs` converts SQL NULL and text `∅` to the same string, joins composite keys with U+001F, splits those keys for WHERE, and decodes `∅` as NULL when generating literals. This can hide changes, reject distinct keys, and generate predicates that target the wrong row or no row. Severity: P1 (incorrect synchronization preview).

## Design and scope
Keep borrowed typed values and structural key tuples inside DataDiffService. Convert only at the existing public string DTO boundary. INSERT and key predicates use typed values; UPDATE uses correctly nullable exact scalar change values (None for real NULL, Some for text), never sentinel/delimiter decoding. Preserve quoted identifiers and escaped literals, precise scalar text, sampling bounds, genuine duplicate rejection and public DTO field types. No SQL-builder/schema-DDL/port/dependency changes.

## Definition of done
- NULL differs from text `∅` in rows and keys; NULL predicates use IS NULL.
- Literal text `∅` survives INSERT and UPDATE.
- Distinct composite tuples containing U+001F compare independently and produce exact predicates.
- Existing added/removed/changed/equal behavior and genuine duplicate detection remain covered.
- Adjacent tests and independent real SQLite/PostgreSQL public-service smoke recipes are handed to Main. Execution and provider evidence must be recorded before closure.

## Tasks
1. Trace callers and existing tests, establish reproducible fixtures.
2. Add regression tests, then replace lossy internal representations and SQL inputs.
3. Record source findings, exact file inventory, selectors, independent provider recipes, and unexecuted gates.


## Source: `FINDINGS.md`

# Findings — #457 / #485

All baseline source assertions below refer to `76b120724a7542de8b587a002811ddacad28156a`; revised behavior describes the uncommitted implementation based on that SHA, not observed execution.

| Severity | Baseline evidence | Consequence / decision |
|---|---|---|
| P1 | `crates/core/src/application/data_diff.rs:275-290,310-320,331-345` converts NULL and text `∅` to identical strings before comparison. | A NULL/text change can be reported equal; key samples can collapse or be rejected as duplicates. Retain typed values, map only actual NULL to DTO `None`. |
| P1 | `crates/core/src/application/data_diff.rs:266-279,310-314` joins/splits U+001F keys, emits `= NULL`, and interprets text `∅` as SQL NULL. | Distinct composite tuples can collide, WHERE can lose tuple boundaries, and literal text becomes NULL. Use structural tuples for identity and predicates; NULL uses IS NULL. |
| P2 | `crates/core/src/application/data_diff.rs:283-290` uses Debug for decimal/temporal/JSON values. | Precision-sensitive strings must not be parsed as f64 or reconstructed from Debug. The replacement retains exact scalar payloads and uses them for comparison/SQL. |

## Callers and compatibility
At baseline `76b120724a7542de8b587a002811ddacad28156a`, `crates/runtime/src/api.rs:1151-1180` and `crates/runtime/src/worker.rs:2612-2619` delegate to the public service and forward its DTO. `crates/app/src/schema/schema_compare_view.rs:498-544` only displays row keys and SQL, replacing U+001F for presentation; it does not decode keys for mutation. `crates/core/src/domain/cross_connection.rs:49-95` defines string-valued DTO fields. No public field/type change is needed. Keys become escaped typed display strings; column-change NULL is `None`, text `∅` remains `Some("∅")`. No caller migration is necessary.

## Deliberate boundaries
- Baseline `76b120724a7542de8b587a002811ddacad28156a`, `crates/core/src/ports/dialect.rs:1-6`, offers no provider-specific literal/binary API. Main explicitly authorized retaining legacy binary preview spelling, with binary literal correctness **not addressed/not verified**. No claim of provider-correct binary synchronization is made.
- Provider-specific MySQL backslash modes, unsupported literal classes, sampling/pagination, capabilities, SQL builders, schema DDL and ports remain unchanged/out of scope. Quoted scalar SQL preserves the existing convention, doubles apostrophes and never reparses decimal/integer values through floating point.
- Float values retain their exact f64 bits for identity; strings/bytes borrow provider values. JSON uses its serialized payload, not a SQL sentinel or composite-key delimiter.
- No source-text/wiring tests, dependencies, inline test modules, commits, pushes or issue closures were added.

## Tooling and evidence
Semantic `find` failed because its judge backend rejected the account; literal symbol searches were used instead. Parent reported LSP unavailable, so literal navigation was retained. Tests and runtime gates were deliberately not executed in this lane; their status remains NOT VERIFIED until Main records actual results.


## Source: `VERIFICATION.md`

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


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — typed data-diff values (#457 / #485)

## 1. Claim
| Field | Value |
|---|---|
| Agent identity | TypedDataDiff / core data-diff lane |
| Issue(s) | #457, #485 |
| Task state | Review |
| Baseline SHA | `76b120724a7542de8b587a002811ddacad28156a` |
| Branch / PR | main / no PR; Main owns integration and publication |
| Scope interpretation | Replace lossy NULL/text and composite-key identity in DataDiffService; generate scalar sync-preview SQL from typed rows. |
| Out of scope | SQL builders, schema DDL, ports, new dependencies, unrelated sampling/provider support, binary literal implementation, MySQL backslash modes, shared changelog/status. |

## 2. Progress checkpoint
- Current integration base: `76b120724a7542de8b587a002811ddacad28156a`; lane changes are uncommitted. No final candidate SHA exists in this lane.
- Implementation acceptance: [x] NULL/text comparison, [x] distinct NULL/text keys, [x] IS NULL predicates, [x] text `∅` INSERT/UPDATE, [x] structural U+001F tuples and exact WHERE boundaries, [x] adjacent regressions and independent provider probes.
- Remaining: [ ] executed focused tests, [ ] SQLite runtime evidence, [ ] PostgreSQL runtime evidence, [ ] integration review/gates/commit/closure.
- Baseline P1 findings at `76b120724a7542de8b587a002811ddacad28156a`: `crates/core/src/application/data_diff.rs:266-290,310-345`; details in FINDINGS.md. Revised implementation not runtime-verified.
- Tests already run: none (explicit lane prohibition). No build/lint/format/cargo execution.
- Dependencies/blockers: parent reported dedicated PostgreSQL fixture and second database provisioning; availability not observed here. No external code changes required.

## 3. Implementation handoff / review request
| Field | Value |
|---|---|
| Exact SHA | Baseline `76b120724a7542de8b587a002811ddacad28156a` + uncommitted lane changes; final SHA must be supplied by Main. |
| Commit list | none |
| File / surface inventory | `crates/core/src/application/data_diff.rs` — borrowed typed values/keys, null predicates, source-row SQL, display-only strings; `crates/core/src/application/tests/data_diff_tests.rs` — updated private typed fixture plus six public-service regressions; `crates/infrastructure/tests/data_diff_smoke.rs` — real SQLite and independent ignored PostgreSQL public-service round trips; this directory's PLAN.md, CHECKLIST.md, FINDINGS.md, VERIFICATION.md and AGENT_EVIDENCE.md — requirements, source evidence and handoff. |
| Acceptance mapping | NULL/text and NULL predicates → null_source/null_keys/empty_set tests; text literal preservation → empty_set test; key collision and intended WHERE → two composite tests; precision/quoting → typed_values test; existing behavior + real SQL execution → existing tests and provider round trips. |
| Commands and counts | No commands executed; exact selectors/recipes in VERIFICATION.md. Counts unavailable, exit status n/a. |
| CI run IDs / status | not run |
| Known limitations | Provider/automated execution unverified; binary literal correctness and MySQL backslash handling are inherited/out of scope. DTO NULL now uses None; keys are escaped typed display strings, never SQL identity. |
| Migrations / config implications | No persisted/public DTO type migration. PostgreSQL smoke requires two distinct disposable writable databases plus PG* credentials and DATA_DIFF_TARGET_DATABASE. |
| Out-of-scope changes | none |

## 4. Review outcome
| Field | Value |
|---|---|
| Reviewed SHA | n/a; no independent review performed |
| Verdict | n/a; request Main review |
| P0 / P1 / P2 counts | Not adjudicated by this lane; baseline findings recorded separately. |
| Findings | Baseline P1 NULL/key failures described in FINDINGS.md; candidate execution pending. |
| CI disposition | all execution gates not run in this lane |
| Next task(s) unblocked | Main's combined verification and provider smoke; close #457/#485 only after fully verified. |

## 5. Research / audit handoff
- Source date: 2026-10-16.
- Source references at baseline `76b120724a7542de8b587a002811ddacad28156a`: core application data_diff.rs and tests/data_diff_tests.rs; domain query.rs/cross_connection.rs; ports dialect.rs/db_connector.rs; runtime api.rs/worker.rs; app schema/schema_compare_view.rs; infrastructure postgres/connector.rs and existing integration/pg_integration.rs conventions.
- Factual findings: baseline string flattening, U+001F join/split and sentinel literal decoding are present; runtime/API forward DTOs and the UI displays keys/SQL. Exact pointers are in FINDINGS.md.
- Inference: baseline public-provider probe should fail on duplicate composite key; independent NULL-only fixture should falsely report equality. Neither reproduction was executed here.
- Decision: borrow typed values internally, use structural tuples, generate SQL from typed source/key data, retain DTO field types; leave provider-specific binary/backslash work untouched as authorized by Main.
- Unresolved questions: no implementation question; runtime/provider readiness and results belong to Main.
- Downstream tasks activated: Main integration/verification for #457/#485.

## 6. Tổng kết (Vietnamese summary)
Đã sửa cụm #457/#485 bằng giá trị có kiểu và khóa tuple: NULL không còn trùng chuỗi `∅`, khóa ghép chứa U+001F không bị gộp, điều kiện NULL dùng IS NULL, SQL lấy dữ liệu gốc thay vì giải mã chuỗi hiển thị. Giữ nguyên kiểu DTO; thêm 6 test hồi quy và smoke độc lập SQLite/PostgreSQL có chạy SQL preview rồi so sánh lại. Chưa chạy bất kỳ gate nào theo phân công; Main cần chạy test/provider smoke, ghi SHA và bằng chứng thật trước khi commit/push/đóng issue. Hành vi binary/MySQL backslash cũ không được mở rộng hoặc xác minh.

