---
schema_version: 1
id: "MEM-SRC-178089297775607d"
title: "Source mirror: docs/plans/active/postgres-autocommit-search-path/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/postgres-autocommit-search-path/FINDINGS.md"
source_sha256: "178089297775607d577d92cf2a48b9597bf1cfb904e5c5e2256f2aae2039f38d"
mirror_body_sha256: "178089297775607d577d92cf2a48b9597bf1cfb904e5c5e2256f2aae2039f38d"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Postgres autocommit search_path — FINDINGS

- Severity: P1 — provider-specific correctness, major user flow broken. Any unqualified table in
  the selected tenant schema failed on PostgreSQL; qualified names still worked, which hides the
  regression from tests that qualify.
- Evidence:
  - Live PostgreSQL reproduction (`set_config(..., true)` in autocommit):
    the next statement reported `"$user", public` as `current_setting('search_path')` and
    `SELECT * FROM m_ctrl_universal_value` failed; the same sequence with `is_local = false`
    returned the tenant row.
  - Connector-level regression test (added, failed before the fix):
    `pg_autocommit_run_resolves_unqualified_tables_in_active_query_schema` failed with
    `QuerySyntaxAt { message: "relation \"scoped_values\" does not exist", position: 13 }`.
- Failure scenario: query tab selects `tenant1`; the run command carries `schema: Some("tenant1")`;
  the connector installs it locally in autocommit; PostgreSQL discards it with the installing
  statement; `SELECT * FROM m_ctrl_universal_value` resolves against `"$user", public` and fails.
- Scope: `crates/infrastructure/src/postgres/search_path.rs` and the query / execute / explain /
  joined-batch call sites in `crates/infrastructure/src/postgres/connector.rs`, plus the session
  lease in `crates/infrastructure/src/transaction_session.rs`.
- Existing coverage: the parameterized-transaction test only exercised an explicit transaction,
  where `SET LOCAL` is correct; no test covered an autocommit query with a schema. The app already
  asserts that the tenant schema is dispatched (`app_tests.rs` tenant RunQuery test), so the loss
  was provider-side.
- Provider impact: PostgreSQL fixed and verified live. SQLite (no schemas beyond `main`), MySQL
  (database-qualified) and SQL Server (default schema) are unaffected by this code path.
- Runtime testability: three live regression tests on the local PostgreSQL 18 fixture; plus unit
  tests for the abandoned/restored lease. Native-app UAT against a real tenant database remains.
