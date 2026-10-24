---
schema_version: 1
id: "MEM-PLAN-empty-schema-qualification-a5eaef16e484"
title: "Plan archive: Empty Schema Qualification"
type: workflow
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, workflow, empty-schema-qualification]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "a5eaef16e484f51f9f8b8fb047ae0cccbfda628ffb7a46a823e61305d0dbc6e1"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/empty-schema-qualification/PLAN.md"
  - "docs/plans/completed/empty-schema-qualification/FINDINGS.md"
  - "docs/plans/completed/empty-schema-qualification/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/empty-schema-qualification/`. Loại suy luận: `workflow`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Empty Schema Qualification Remediation — PLAN

State: COMPLETED

## Objective
Fix invalid SQL formatting when table or object `schema` is an empty string `""` across core query builders (`sql_builder.rs`), DDL generators (`schema_service.rs`), data diffs (`data_diff.rs`), schema diffs (`schema_diff.rs`), and cross-connection utilities (`cross_connection.rs`).

## Problem Statement
When `schema` is `""` (common in SQLite databases or default un-scoped connections), query builders format `"{schema}.{table}"` directly (e.g., `""."table_name"`). Both SQLite and PostgreSQL reject this as a syntax error (`zero-length delimited identifier` or `syntax error near "."`).

## Scope
1. Core SQL Builder (`crates/core/src/application/sql_builder.rs`):
   - Add canonical `qualify(dialect, schema, table)` helper function.
   - Refactor `build_select`, `build_count`, `build_insert`, `build_update`, `build_delete` to use `qualify`.

2. Schema Service (`crates/core/src/application/schema_service.rs`):
   - Handle empty `schema` and `to_schema` when constructing table qualification and FK target qualification in DDL generation.

3. Data Diff (`crates/core/src/application/data_diff.rs`):
   - Handle empty `schema` when building source and target table qualifications.

4. Schema Diff (`crates/core/src/application/schema_diff.rs`):
   - Handle empty `schema` in table and index qualified name generation and parsing.

5. Cross Connection (`crates/infrastructure/src/postgres/cross_connection.rs`):
   - Handle empty `schema` in `rename_schema_object`.

6. Automated Tests:
   - Unit tests for empty `schema = ""` across all builders.
   - Core and infrastructure test suites execution.


## Source: `FINDINGS.md`

# Defect Finding: Invalid SQL Generation on Empty Schema Qualification

## Severity: P1 (Wrong database mutation / Unsafe SQL / Provider capability mismatch)

### Area
- Core SQL Query Building (`sql_builder.rs`)
- DDL Generation (`schema_service.rs`)
- Data & Schema Diffing (`data_diff.rs`, `schema_diff.rs`)
- PostgreSQL Object Operations (`cross_connection.rs`)

### Code Evidence
In `crates/core/src/application/sql_builder.rs`:
```rust
let sql = format!(
    "SELECT * FROM {}.{}{}{}{pagination}",
    dialect.quote_identifier(schema),
    dialect.quote_identifier(table),
    where_clause,
    order_clause,
);
```
When `schema` is `""`, `dialect.quote_identifier("")` outputs `""`. The resulting query is `SELECT * FROM ""."table_name"`.

Similar unconditional formatting exists in:
- `build_count` (`sql_builder.rs`)
- `build_insert` (`sql_builder.rs`)
- `build_update` (`sql_builder.rs`)
- `build_delete` (`sql_builder.rs`)
- `build_create_table_ddl` (`schema_service.rs`)
- `diff_table_data` (`data_diff.rs`)
- `rename_schema_object` (`cross_connection.rs`)

### Failure Scenario
1. User interacts with an SQLite table or database where `schema` is empty `""`.
2. Data grid queries (select/count/insert/update/delete) or DDL actions execute `sql_builder` or `schema_service` methods.
3. The generated SQL includes `""."table_name"`.
4. The database driver rejects the statement with a syntax error (`zero-length delimited identifier` in PostgreSQL or `syntax error near "."` in SQLite).
5. The operation fails completely, preventing row display, editing, deletion, insertion, DDL view, and diffing for un-scoped tables.

### Required Fix
Provide safe qualification logic (`qualify(dialect, schema, table)`) that omits `schema.` when `schema.is_empty()`.


## Source: `VERIFICATION.md`

# Empty Schema Qualification — VERIFICATION

## Test Strategy

1. **Unit Verification (`crates/core/src/application/sql_builder.rs`)**:
   - Test `build_select`, `build_count`, `build_insert`, `build_update`, `build_delete` with `schema = ""`.
   - Assert generated SQL contains `"users"` (or `` `users` ``) without `""."users"`.

2. **Unit Verification (`crates/core/src/application/schema_service.rs`)**:
   - Test `build_create_table_ddl` with `schema = ""`.
   - Assert generated CREATE TABLE SQL does not contain `""."tablename"`.

3. **Unit Verification (`crates/core/src/application/data_diff.rs` & `schema_diff.rs`)**:
   - Test data diff and schema diff when `schema` is `""`.

4. **Integration Gate Execution**:
   - `cargo test -p db-pro-core -p db-pro-infrastructure`

