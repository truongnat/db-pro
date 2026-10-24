---
schema_version: 1
id: "MEM-SRC-1c697d90ba48a257"
title: "Source mirror: docs/plans/active/issue-477-backslash-escape-lexing/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-477-backslash-escape-lexing/PLAN.md"
source_sha256: "1c697d90ba48a257c0ce77e461ef72aa38ce091bbf943ea595b4f070cf338d81"
mirror_body_sha256: "1c697d90ba48a257c0ce77e461ef72aa38ce091bbf943ea595b4f070cf338d81"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
