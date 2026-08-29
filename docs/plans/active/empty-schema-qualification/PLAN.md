# Empty Schema Qualification Remediation — PLAN

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
