# Verification — Query Editor Hover Intelligence

## Quality Gates Execution Summary

| Gate | Command | Result | Evidence |
|------|---------|--------|----------|
| Rust Format | `cargo fmt --all -- --check` | **PASS** | Exit code 0, 0 format violations |
| Rust Check | `cargo check --workspace` | **PASS** | Exit code 0, 0 errors, 0 warnings |
| Rust Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | **PASS** | Exit code 0, 0 lints |
| Workspace Tests | `cargo test --workspace` | **PASS** | 548 passed; 0 failed; 0 ignored |
| Native Release Build | `cargo build --release -p db-pro-native` | **PASS** | Compiled in release profile |

## Unit & Integration Test Coverage

### 1. Hover FSM Delay & Lifecycle Tests (`crates/ui/src/editor/hover.rs`)
- `hover_confirmed_after_delay`: Validates that hovering on a token requires $\ge 500\text{ ms}$ before confirming.
- `hover_cleared_on_pointer_leave`: Validates that moving the mouse away immediately clears the confirmed/pending token.
- `hover_resets_on_different_token`: Validates that moving across different tokens resets the delay timer and avoids stale popup rendering.
- `pending_repaint_returns_remaining_time`: Validates that egui is requested to repaint at the exact time the delay elapses.

### 2. Rich Hover Tests (`crates/ui/src/query/intelligence.rs`)
- `rich_hover_resolves_table_columns_and_foreign_keys`: Resolves a table hover card with full column list, data types, primary key flags, foreign key relationships, and row count.
- `rich_hover_resolves_qualified_column_with_fk_target`: Resolves a column hover card showing data type, primary key flag, parent table, and target foreign key reference `public.users(id)`.
- `rich_hover_keyword_returns_documentation_and_dialect_note`: Resolves SQL keywords (e.g. `SELECT`, `RETURNING`, `PRAGMA`) with dialect note (`PostgreSQL` / `SQLite`), explanation, and example SQL snippet.
- `signature_help_prefers_introspected_active_schema_function`: Resolves active schema function signature and parameter context.
- `sqlite_and_postgres_builtins_remain_provider_specific`: Ensures dialect-specific functions are only available on matching database targets.

## Provider Matrix Coverage

| Feature | PostgreSQL | SQLite | Verification Method |
|---------|------------|--------|---------------------|
| Table hover (columns, row count, FKs) | Supported | Supported | Unit tests + schema summary model |
| Column hover (type, nullability, PK, FK) | Supported | Supported | Unit tests |
| Keyword hover (DML, DDL, clauses, examples) | Supported | Supported | Unit tests with PostgreSQL & SQLite dialects |
| Function signature hover | Supported (schema + built-ins) | Supported (built-ins) | Unit tests |
