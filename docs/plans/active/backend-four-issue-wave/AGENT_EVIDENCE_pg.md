# Agent evidence — PostgreSQL #499 / #508

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | PgBuilderDependencies |
| Issue(s) | #499, #508 |
| Task state | Ready |
| Baseline SHA | bceb735727f81180dad678defee1000bc547d97b |
| Branch / PR | main; no child commit/push or PR |
| Scope interpretation | Fix simple event-trigger function reference generation/validation and exact outgoing PostgreSQL relation dependency identity; supply adjacent validation and isolated real-provider regressions. |
| Out of scope | Other providers, CSV, shared CHANGELOG/STATUS, dependencies/manifests, confirmation redesign, reverse-dependency redesign, gate execution, publishing/issue closure. |

## 2. Progress checkpoint

- Source baseline supplied by Main: `bceb735727f81180dad678defee1000bc547d97b`. Candidate is an uncommitted patch based on that source; no candidate SHA or current-HEAD observation is claimed.
- [x] R1 implementation: optional empty parentheses, exact simple identifier components, lowercase-before-quote semantics, mandatory generated `()`.
- [x] R2 implementation: exact separately bound schema/relation names, catalog class/OID join, normal/auto outgoing semantics preserved, native identity for non-relation/non-function targets, empty missing-name result.
- [x] Permanent validation regressions and two opt-in real PostgreSQL consumer-visible tests written.
- [x] Consumer trace, authoritative grammar/catalog sources, exact smoke calls and fixture requirements documented in `PG_FINDINGS.md`.
- [ ] D1/D2 runtime outcomes, before/after baseline probes, all gates, final independent review, commits/pushes and issue closure: Main-owned, **NOT RUN** here.
- Baseline findings: #499 whitelist accepts malformed references and missing invocation parentheses (`event_trigger.rs:59-70,87-106` at baseline SHA); #508 displays/pattern-matches names and attempts row-field access on an OID alias (`cross_connection.rs:10-45` at baseline SHA). Runtime reproduction is not claimed.
- Dependency/blocker changes: no implementation blocker. Main's gate ownership is unchanged. No P0/P1 runtime defect in the candidate has been observed because it has not been executed.
- Tests already run: **NOT RUN**; pass/fail/ignored counts and exits n/a by explicit assignment.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | Baseline bceb735727f81180dad678defee1000bc547d97b; candidate uncommitted, no candidate SHA yet |
| Commit list | none — Main owns individual issue commits |
| File / surface inventory | `crates/core/src/domain/event_trigger.rs`: component validation + normalized quoted invocation. `crates/core/src/domain/tests/event_trigger_tests.rs`: accepted/malformed/length validation cases; necessary existing preview expectation. `crates/infrastructure/src/postgres/cross_connection.rs`: exact catalog identity query and native target identities. `crates/infrastructure/tests/pg_object_identity.rs`: new isolated real-PG tests. `docs/plans/active/backend-four-issue-wave/PG_FINDINGS.md`: detailed findings, sources, calls, expected rows and fixture contract. This evidence file: canonical handoff. |
| Acceptance mapping | #499 → validator/builder plus `pg_event_trigger_function_refs_create_fire_and_introspect`; #508 → class/OID-bound lookup plus `pg_relation_dependencies_use_exact_catalog_identity`; unit validation regressions cover malformed/arg-bearing names without new SQL-string assertions. |
| Commands and counts | **NOT RUN**: `cargo test -p db-pro-core domain::event_trigger::tests`; `DATABASE_URL=... cargo test -p db-pro-infrastructure --test pg_object_identity -- --ignored`; formatting/check/Clippy/workspace tests/release build/runtime probes. All counts/exits n/a. |
| CI run IDs / status | NOT RUN |
| Known limitations | Runtime/gate outcomes unobserved. Opt-in provider tests require disposable PG; event-trigger test requires superuser. Fixture parser follows existing plain local postgres:// credential convention, not arbitrary URL escaping. Production function input intentionally stays the existing simple ASCII identifier subset. Outgoing relation dependencies do not traverse rewrite/constraint/trigger source catalogs. |
| Migrations / config implications | None in production. Tests read DATABASE_URL; successful runs remove UUID-named fixture objects, failed runs may leave objects in the disposable database. Public schema must be visible in the fixture's default search path. |
| Out-of-scope changes | none |

Exact focused selectors:

- Core: `domain::event_trigger::tests::accepts_simple_zero_argument_function_references`
- Core: `domain::event_trigger::tests::rejects_arguments_and_malformed_function_references`
- Core: `domain::event_trigger::tests::rejects_overlong_function_identifier_components`
- Infrastructure test target `pg_object_identity`: `pg_event_trigger_function_refs_create_fire_and_introspect`
- Infrastructure test target `pg_object_identity`: `pg_relation_dependencies_use_exact_catalog_identity`

Public smoke interfaces: `PostgresEventTriggerPort::{create_event_trigger, inventory, alter_event_trigger, drop_event_trigger}` and `DbConnector::execute`; `PostgresConnector::get_object_dependencies` (runtime consumer `PostgresApi::object_dependencies`). Exact argument examples and expected complete rows are in `PG_FINDINGS.md`. Both production fixes are ready for Main's stable integration and provider gates; edits stop here.

## 4. Review outcome

n/a — this is an implementation handoff, not an independent review. No ACCEPT/BLOCK verdict or introduced P0/P1/P2 count is claimed. Reviewed candidate SHA and CI disposition await Main's committed/gated candidate. Baseline findings are separated above from unobserved candidate runtime outcomes. Next tasks unblocked: Main's #499/#508 baseline/candidate probes, provider/unit gates, individual commit/publish/closure decisions.

## 5. Research / audit handoff

- Source date: 2026-10-16.
- Repository references at baseline `bceb735727f81180dad678defee1000bc547d97b`: core event trigger builder/adjacent tests; app `tools/event_trigger_activity_view.rs:59,73`; infra `postgres/event_trigger.rs:87-137`; infra dependency helper and `connector.rs:1002-1017`; runtime `api.rs:1211-1222`; core `domain/cross_connection.rs:98-104`; adjacent provider integration fixture conventions.
- Official sources: https://www.postgresql.org/docs/17/sql-createeventtrigger.html ; https://www.postgresql.org/docs/17/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS ; https://www.postgresql.org/docs/17/catalog-pg-depend.html ; https://www.postgresql.org/docs/17/functions-info.html#FUNCTIONS-INFO-OBJECT-TABLE ; https://raw.githubusercontent.com/postgres/postgres/REL_17_STABLE/src/backend/catalog/objectaddress.c .
- Factual findings: official grammar requires empty invocation parentheses; unquoted identifiers fold lowercase; pg_depend source identity includes catalog class and object OID; pg_identify_object exposes nontranslated identities and avoids interpreting unrelated object IDs as relations. Namespace identity quotes as needed and type identity is schema-qualified in the official implementation.
- Inference: the candidate should repair the reported real-PG failures; not reported as runtime evidence until Main executes the exported-baseline and candidate probes.
- Decision/recommendation: reuse core validation/quoting with one bounded function-reference grammar; exact catalog-bound identity with unchanged dependency direction; no generic parser, LIKE escaping, raw SQL names/literals or new dependency. Numeric nonrelation target fallback becomes native machine-readable identity.
- Unresolved questions: no source-level design question; actual fixture/gate outcomes await Main.
- Downstream tasks activated: #499/#508 runtime validation, review and individual publishing/closure by Main.

## 6. Tổng kết (Vietnamese summary)

Đã sửa bộ dựng event trigger và tra cứu dependency PostgreSQL theo đúng danh tính catalog; bổ sung kiểm tra đầu vào và hai test PG thật, kèm hướng dẫn fixture/call/đầu ra. Trạng thái Ready; chưa chạy bất kỳ gate hay runtime probe nào theo phân công. Main cần chạy baseline/candidate, kiểm tra độc lập rồi commit/push/đóng từng issue; agent không sửa thêm sau bàn giao.
