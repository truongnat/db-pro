---
schema_version: 1
id: "MEM-PLAN-issue-477-backslash-escape-lexin-3971423c6444"
title: "Plan archive: Issue 477 Backslash Escape Lexing"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-477-backslash-escape-lexing]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "3971423c64441ab15bc99c07735fe60bc0042fe19dfa2ab2c6c22ffa38dbb034"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-477-backslash-escape-lexing/PLAN.md"
  - "docs/plans/active/issue-477-backslash-escape-lexing/FINDINGS.md"
  - "docs/plans/active/issue-477-backslash-escape-lexing/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-477-backslash-escape-lexing/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# #477 — Statement splitter must honor dialect string escapes

## Goal
`skip_quoted` treated `\` as an escape inside every `'…'` literal. With
PostgreSQL `standard_conforming_strings=on` (default), `'x\'` is the string
`x\` plus a closing quote, so `SELECT 'x\'; DROP TABLE important; --` split as
ONE statement and `reject_multi_statement` let the destructive tail through.

## Scope
- `crates/core/src/domain/safety_lexer.rs`: `skip_quoted` gains an
  `escapes_enabled` flag; `special_token_end`, `matching_parenthesis_end`,
  `tokenize_sql` take `backslash_escapes`; new `is_escape_string_start`
  detects the `E'…'`/`e'…'` prefix (always escape-lexed).
- `crates/core/src/domain/safety.rs`: `split_statements_for_driver`,
  `classify_statement_safety_for_driver`, `classify_script_safety_for_driver`;
  one-arg public wrappers default to `DriverType::Postgres` (standard strings,
  the conservative reading). Private helpers thread `backslash_escapes`.
- `crates/core/src/domain/safety_cte.rs`: `classify_cte_safety(sql, escapes)`.
- `crates/core/src/application/sql_policy.rs`:
  `reject_multi_statement(sql, driver)`.
- `crates/core/src/application/query_service.rs`: new `driver_for` helper;
  `execute`, `explain`, `execute_multi` pass the driver; per-statement
  classification uses `classify_statement_safety_for_driver` /
  `classify_statement(statement, driver)`.
- `crates/core/src/application/multi_query_execution.rs`: `driver` field so
  `execute_sequential` classification matches the split.
- `crates/core/src/application/export_service.rs`: config fetched before the
  gate; `reject_multi_statement(sql, config.driver)`.
- `crates/core/src/application/schema_service.rs`: `execute_ddl` /
  `execute_ddl_batch` split with `split_statements_for_driver`.
- Tests: `domain/safety/tests.rs` (#477 regression block),
  `application/tests/sql_policy_tests.rs`, `application/query_service/tests.rs`
  (driver arg on `classify_statement`, `build_service` gets a config mock).

## Non-goals
- `validate_against_policy` keeps the standard-strings default. Standard
  lexing is conservative (hides nothing), so read-only gates stay safe;
  passing a driver there would only tune MySQL false positives.
- `has_top_level_sql_keyword` gained an `escapes` param (callers all supply
  it); `transaction_control_verb` inspects only the leading keyword, unchanged.
- App/runtime callers of `classify_script_safety`/`split_statements` keep the
  Postgres default — correct for PG/SQLite/SQL Server and conservative for
  MySQL.

## Acceptance
- `split_statements("SELECT 'x\\'; SELECT 2")` → 2 on PG; 1 on MySQL.
- `SELECT E'x\''; SELECT 2` → 2 (escape literal closes after `\'`).
- `cargo test -p db-pro-core` green; `cargo check --workspace` green.


## Source: `FINDINGS.md`

# #477 findings

- Root cause: `safety_lexer::skip_quoted` unconditionally skipped `\x` inside
  `'…'` for every dialect. PG default (`standard_conforming_strings=on`) ends
  `'x\'` at the quote, so `SELECT 'x\'; DROP TABLE t; --` was ONE statement to
  the splitter and `reject_multi_statement` passed it.
- The same lexer feeds `classify_*`, `top_level_where_clause`,
  `contains_sql_keyword`, `matching_parenthesis_end`, `tokenize_sql` — all now
  take the escape flag so classification and splitting agree per dialect.
- `E'…'` literals opt into escapes on their own; detected by `is_escape_string_start`
  which requires the `E` not to be glued to an identifier (`someE'…'` is an
  identifier + plain string).
- Driver plumbing: `ConnectionConfig.driver` is the canonical source.
  `QueryService::driver_for` fetches it once per entry point; ExportService
  already fetched config (moved above the gate); SchemaService fetches per
  call. `backslash_escapes_for_driver` maps only `DriverType::Mysql` to true.
- Default-safe API: `split_statements`, `classify_statement_safety`,
  `classify_script_safety`, `validate_against_policy` keep the Postgres
  (standard-strings) reading — correct everywhere except MySQL, where it can
  only over-split/over-classify, never hide a statement.
- Behavior change worth noting: `execute`/`explain`/`execute_for_export` now
  hit the connection repo before the multi-statement gate; an unknown
  connection reports "not found" rather than "empty SQL statement". Test
  helpers (`build_service`) got a `get_config` mock accordingly.


## Source: `VERIFICATION.md`

# #477 verification

## Checks run
- `cargo check -p db-pro-core` — clean (after fixing signature fallout in
  `multi_query_execution.rs`, `query_service/tests.rs`).
- `cargo test -p db-pro-core` — 502 passed, 0 failed.
- `cargo check --workspace` — clean (db-pro-core, infrastructure, ui, app,
  runtime, native-app).

## New regression coverage
- `safety/tests.rs`
  - `split_statements_postgres_backslash_does_not_escape_quote` —
    `SELECT 'x\'; SELECT 2` → 2 statements (default, Postgres, SQLite,
    SqlServer drivers)
  - `split_statements_mysql_backslash_escapes_quote` — same text → 1
    statement on MySQL; `'it\'s ok'` handled
  - `split_statements_escape_string_literal` — `E'x\''`/`e'a\'b'` split
    correctly; `someE'x\'` stays a single identifier+string run
  - `script_safety_sees_statements_after_a_closed_backslash_string` —
    `SELECT 'x\'; DROP TABLE important; --` → Destructive (PG), Read (MySQL)
- `sql_policy_tests.rs`
  - `reject_multi_statement_catches_pg_backslash_before_quote` —
    the exact issue-477 exploit string is rejected on PG
  - `reject_multi_statement_keeps_mysql_backslash_escape` — accepted on MySQL
- `query_service/tests.rs` — all `classify_statement` calls updated to
  `classify_statement(sql, DriverType::Postgres)`; `build_service` now uses
  `mock_connections_full_access()` since `execute` consults the config for the
  driver before the gate.

